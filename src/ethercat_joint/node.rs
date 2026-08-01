//! Wire robot-bus Node: JointCommand in → EtherCAT → JointState / diagnostics out.

use super::cia402::DriveState;
use super::config::{BackendKind, EthercatJointConfig};
use super::hooks::{FaultAction, JointHooks, NoopHooks};
use super::mapping::{
    any_fault, build_setpoints, feedback_to_joint_state, CommandCache,
};
use super::master::{create_master, EthercatMaster, JointFeedback};
use anyhow::{Context, Result};
use crate::diagnostic_msgs::msg::v1::{DiagnosticArray, DiagnosticStatus, KeyValue};
use crate::robot_bus_interface::msg::v1::JointCommand;
use crate::sensor_msgs::msg::v1::JointState;
use crate::std_srvs::srv::v1::{
    SetBool, SetBoolRequest, SetBoolResponse, Trigger, TriggerRequest, TriggerResponse,
};
use crate::{Node, NodeOptions, TopicPublisher};
use prost::Message;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct SharedPub<T: Message + Default> {
    inner: Mutex<TopicPublisher<T>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl<T: Message + Default> Send for SharedPub<T> {}
unsafe impl<T: Message + Default> Sync for SharedPub<T> {}

struct SharedState {
    cache: CommandCache,
    want_enabled: bool,
    pulse_fault_reset: bool,
    last_command_at: Option<Instant>,
}

/// Run with default no-op hooks (CLI entrypoint).
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    run_with_hooks(node_name, options, params_path, NoopHooks)
}

/// Run with custom [`JointHooks`] for secondary development.
pub fn run_with_hooks<H: JointHooks + 'static>(
    node_name: &str,
    options: NodeOptions,
    params_path: Option<&str>,
    mut hooks: H,
) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = EthercatJointConfig::load(&mut node, params_path)?;

    log::info!(
        "ethercat_joint: backend={:?} iface={} joints={} cycle={}ns -> {} <- {}",
        cfg.backend,
        cfg.iface,
        cfg.joints.len(),
        cfg.cycle_ns,
        cfg.output_topic,
        cfg.command_topic
    );

    let mut master = create_master(cfg.backend, &cfg.iface)?;
    master.configure(&cfg.joints)?;
    if cfg.auto_enable {
        master.set_want_enabled(true);
    }

    let state_pub = Arc::new(SharedPub {
        inner: Mutex::new(
            node.create_publisher::<JointState>(&cfg.output_topic)
                .context("create JointState publisher")?,
        ),
    });
    let diag_pub = Arc::new(SharedPub {
        inner: Mutex::new(
            node.create_publisher::<DiagnosticArray>(&cfg.diagnostics_topic)
                .context("create DiagnosticArray publisher")?,
        ),
    });

    let shared = Arc::new(Mutex::new(SharedState {
        cache: CommandCache::new(cfg.joints.len()),
        want_enabled: cfg.auto_enable,
        pulse_fault_reset: false,
        last_command_at: None,
    }));

    let (cmd_tx, cmd_rx) = mpsc::channel::<JointCommand>();
    let command_topic = cfg.command_topic.clone();
    node.create_subscription::<JointCommand, _>(
        &command_topic,
        move |_topic, msg| {
            if cmd_tx.send(msg).is_err() {
                log::debug!("joint command channel closed");
            }
        },
        None,
    )
    .context("create JointCommand subscription")?;

    // Optional enable / fault-reset services (Phase 3).
    let enable_shared = Arc::clone(&shared);
    let enable_master_flag = Arc::new(Mutex::new(())); // sync point only; master lives in cycle thread
    let _enable_svc = node.create_service::<SetBool, _>(
        &cfg.enable_service,
        {
            let shared = Arc::clone(&enable_shared);
            move |req: SetBoolRequest| {
                if let Ok(mut st) = shared.lock() {
                    st.want_enabled = req.data;
                    SetBoolResponse {
                        success: true,
                        message: if req.data {
                            "enable requested".into()
                        } else {
                            "disable requested".into()
                        },
                    }
                } else {
                    SetBoolResponse {
                        success: false,
                        message: "state lock poisoned".into(),
                    }
                }
            }
        },
        None,
    );
    let _ = enable_master_flag;

    let fault_shared = Arc::clone(&shared);
    let _fault_svc = node.create_service::<Trigger, _>(
        &cfg.fault_reset_service,
        move |_req: TriggerRequest| {
            if let Ok(mut st) = fault_shared.lock() {
                st.pulse_fault_reset = true;
                TriggerResponse {
                    success: true,
                    message: "fault reset pulsed".into(),
                }
            } else {
                TriggerResponse {
                    success: false,
                    message: "state lock poisoned".into(),
                }
            }
        },
        None,
    );

    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);
    let shared_thread = Arc::clone(&shared);
    let state_pub_thread = Arc::clone(&state_pub);
    let diag_pub_thread = Arc::clone(&diag_pub);
    let joints = cfg.joints.clone();
    let cycle = Duration::from_nanos(cfg.cycle_ns);
    let state_period = Duration::from_secs_f64(1.0 / f64::from(cfg.state_rate_hz));
    let command_timeout = if cfg.command_timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(u64::from(cfg.command_timeout_ms)))
    };
    let frame_id = cfg.frame_id.clone();

    let join = thread::Builder::new()
        .name("ethercat-joint".into())
        .spawn(move || {
            if let Err(e) = cycle_loop(
                stop_thread,
                master,
                joints,
                cycle,
                state_period,
                command_timeout,
                frame_id,
                shared_thread,
                cmd_rx,
                state_pub_thread,
                diag_pub_thread,
                &mut hooks,
            ) {
                log::error!("ethercat joint cycle thread exited: {e:#}");
            }
        })
        .context("spawn ethercat joint cycle thread")?;

    let spin_result = node.spin().context("node spin");
    stop.store(true, Ordering::SeqCst);
    if let Err(e) = join.join() {
        log::warn!("ethercat joint thread join: {e:?}");
    }
    spin_result
}

fn cycle_loop<H: JointHooks>(
    stop: Arc<AtomicBool>,
    mut master: Box<dyn EthercatMaster>,
    joints: Vec<super::config::JointConfig>,
    cycle: Duration,
    state_period: Duration,
    command_timeout: Option<Duration>,
    frame_id: String,
    shared: Arc<Mutex<SharedState>>,
    cmd_rx: mpsc::Receiver<JointCommand>,
    state_pub: Arc<SharedPub<JointState>>,
    diag_pub: Arc<SharedPub<DiagnosticArray>>,
    hooks: &mut H,
) -> Result<()> {
    let mut feedback = vec![JointFeedback::default(); joints.len()];
    let mut next_state_pub = Instant::now();
    let mut next_cycle = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        // Drain commands.
        loop {
            match cmd_rx.try_recv() {
                Ok(raw) => {
                    let cmd = hooks.on_command(raw);
                    if let Ok(mut st) = shared.lock() {
                        st.cache.apply_command(&joints, &cmd);
                        st.last_command_at = Some(Instant::now());
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }

        let now = Instant::now();
        if now < next_cycle {
            let sleep_for = (next_cycle - now).min(Duration::from_millis(1));
            thread::sleep(sleep_for);
            continue;
        }
        next_cycle = now + cycle;

        let (want_enabled, pulse_reset, timed_out) = {
            let mut st = shared.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(timeout) = command_timeout {
                if let Some(t0) = st.last_command_at {
                    if t0.elapsed() > timeout && st.want_enabled {
                        log::warn!("joint command timeout ({timeout:?}); disabling");
                        st.want_enabled = false;
                    }
                }
            }
            let timed_out = command_timeout.is_some()
                && st
                    .last_command_at
                    .map(|t| t.elapsed() > command_timeout.unwrap())
                    .unwrap_or(false);
            let pulse = st.pulse_fault_reset;
            st.pulse_fault_reset = false;
            master.set_want_enabled(st.want_enabled);
            (st.want_enabled, pulse, timed_out)
        };

        let setpoints = {
            let st = shared.lock().unwrap_or_else(|e| e.into_inner());
            build_setpoints(&joints, &st.cache, &feedback, want_enabled, pulse_reset)
        };

        if let Err(e) = master.cycle(&setpoints, &mut feedback) {
            log::error!("master cycle failed: {e:#}");
            publish_diag(
                &diag_pub,
                &joints,
                &feedback,
                &frame_id,
                2,
                &format!("cycle error: {e:#}"),
            );
            continue;
        }

        // Fault hooks.
        for (i, fb) in feedback.iter().enumerate() {
            if DriveState::from_statusword(fb.statusword) == DriveState::Fault {
                match hooks.on_fault(&joints[i].name, fb.statusword) {
                    FaultAction::Reset => {
                        if let Ok(mut st) = shared.lock() {
                            st.pulse_fault_reset = true;
                        }
                    }
                    FaultAction::Disable => {
                        if let Ok(mut st) = shared.lock() {
                            st.want_enabled = false;
                        }
                        master.set_want_enabled(false);
                    }
                    FaultAction::Ignore => {}
                }
            }
        }

        if Instant::now() >= next_state_pub {
            next_state_pub = Instant::now() + state_period;
            let (sec, nsec) = stamp_now();
            let msg = feedback_to_joint_state(&joints, &feedback, &frame_id, sec, nsec);
            if let Err(e) = publish_msg(&state_pub, &msg) {
                log::warn!("publish JointState failed: {e:#}");
            }

            let level = if any_fault(&feedback) {
                2
            } else if timed_out {
                1
            } else {
                0
            };
            let message: &str = if any_fault(&feedback) {
                "fault"
            } else if timed_out {
                "command timeout"
            } else {
                "ok"
            };
            publish_diag(&diag_pub, &joints, &feedback, &frame_id, level, message);
        }
    }

    master.shutdown();
    Ok(())
}

fn publish_msg<T: Message + Default>(pub_: &SharedPub<T>, msg: &T) -> Result<()> {
    let p = pub_.inner.lock().unwrap_or_else(|e| e.into_inner());
    p.publish(msg).context("publish")?;
    Ok(())
}

fn publish_diag(
    pub_: &Arc<SharedPub<DiagnosticArray>>,
    joints: &[super::config::JointConfig],
    feedback: &[JointFeedback],
    frame_id: &str,
    level: u32,
    message: &str,
) {
    let (sec, nsec) = stamp_now();
    let mut status = Vec::new();
    for (i, j) in joints.iter().enumerate() {
        let fb = feedback.get(i).copied().unwrap_or_default();
        status.push(DiagnosticStatus {
            level,
            name: j.name.clone(),
            message: message.to_string(),
            hardware_id: format!("{:#06x}", j.station_address),
            values: vec![
                KeyValue {
                    key: "statusword".into(),
                    value: format!("{:#06x}", fb.statusword),
                },
                KeyValue {
                    key: "mode".into(),
                    value: j.mode.as_str().into(),
                },
                KeyValue {
                    key: "online".into(),
                    value: fb.online.to_string(),
                },
            ],
        });
    }
    let arr = DiagnosticArray {
        header: Some(crate::std_msgs::msg::v1::Header {
            stamp: Some(crate::builtin_interfaces::msg::v1::Time {
                sec,
                nanosec: nsec,
            }),
            frame_id: frame_id.to_string(),
        }),
        status,
    };
    if let Err(e) = publish_msg(pub_, &arr) {
        log::warn!("publish diagnostics failed: {e:#}");
    }
}

fn stamp_now() -> (i32, u32) {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (dur.as_secs() as i32, dur.subsec_nanos())
}

/// List devices for `--list-devices` (mock lists configured joints only after configure;
/// without params, probe ethercrab scan is best-effort via temporary mock message).
pub fn list_devices(params_path: Option<&str>) -> Result<()> {
    match params_path {
        Some(path) => {
            let mut node = Node::with_options("ethercat_list", NodeOptions::inproc());
            let cfg = EthercatJointConfig::load(&mut node, Some(path))?;
            let mut master = create_master(cfg.backend, &cfg.iface)?;
            if cfg.backend == BackendKind::Mock {
                master.configure(&cfg.joints)?;
            } else {
                // Real scan: configure discovers the bus.
                master.configure(&cfg.joints)?;
            }
            for s in master.list_slaves() {
                println!(
                    "{:#06x}\t{}",
                    s.configured_address,
                    s.name
                );
            }
            master.shutdown();
            Ok(())
        }
        None => {
            eprintln!(
                "rbus_ethercat_joint --list-devices requires --params so iface/backend/joints are known"
            );
            anyhow::bail!("missing --params for --list-devices");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ethercat_joint::config::JointMode;
    use crate::ethercat_joint::master::MockMaster;
    use crate::NodeOptions;

    #[test]
    fn mock_node_publishes_after_command() {
        let dir = std::env::temp_dir().join(format!("ec_node_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("p.yaml");
        let yaml = r#"
ros__parameters:
  iface: lo
  backend: mock
  cycle_ns: 1000000
  state_rate_hz: 50
  command_timeout_ms: 0
  auto_enable: true
  output_topic: /joint_states
  command_topic: /joint_commands
  diagnostics_topic: /diagnostics
  enable_service: /ec/enable
  fault_reset_service: /ec/fault_reset
  frame_id: test
  joints:
    - name: joint_1
      station_address: 1001
      mode: csp
      encoder_ticks_per_rev: 1000
      gear_ratio: 1.0
      direction: 1
"#;
        std::fs::write(&path, yaml).unwrap();

        let mut node = Node::with_options("ec_cfg", NodeOptions::inproc());
        let cfg = EthercatJointConfig::load(&mut node, Some(path.to_str().unwrap())).unwrap();
        assert_eq!(cfg.backend, BackendKind::Mock);
        assert_eq!(cfg.joints[0].mode, JointMode::Csp);

        let mut master = MockMaster::new();
        master.configure(&cfg.joints).unwrap();
        master.set_want_enabled(true);
        let mut fb = vec![JointFeedback::default(); 1];
        let cache = CommandCache::new(1);
        let sps = build_setpoints(&cfg.joints, &cache, &fb, true, false);
        master.cycle(&sps, &mut fb).unwrap();
        // After a few enable steps
        for _ in 0..4 {
            let sps = build_setpoints(&cfg.joints, &cache, &fb, true, false);
            master.cycle(&sps, &mut fb).unwrap();
        }
        assert_eq!(DriveState::from_statusword(fb[0].statusword), DriveState::OperationEnabled);
    }
}
