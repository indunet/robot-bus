//! Wire robot-bus Node: USB Xbox pad → `XboxJoy` out, rumble in.

use super::config::JoyConfig;
use super::device::resolve_joy_id;
use super::mapping::to_xbox_joy;
use super::rumble::{apply_rumble, ActiveRumble};
use anyhow::{Context, Result};
use gilrs::{EventType, GamepadId, Gilrs};
use crate::robot_bus_interface::msg::v1::{XboxJoy, XboxJoyRumble};
use crate::{Node, NodeOptions, TopicPublisher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Mutex-guarded publisher shared with the gilrs poll thread.
struct SharedPub {
    inner: Mutex<TopicPublisher<XboxJoy>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedPub {}
unsafe impl Sync for SharedPub {}

/// Build the node, poll the pad on a background thread, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = JoyConfig::load(&mut node, params_path)?;

    // Probe once so startup fails fast when gilrs cannot init.
    {
        let probe = Gilrs::new().map_err(|e| anyhow::anyhow!("init gilrs: {e}"))?;
        match resolve_joy_id(&probe, &cfg.device) {
            Ok(id) => {
                let name = probe.gamepad(id).name().to_string();
                let ff = probe.gamepad(id).is_ff_supported();
                log::info!(
                    "xbox joy selected: id={id} name={name:?} ff_supported={ff} -> {} (rumble <- {}) @ {} Hz",
                    cfg.output_topic,
                    cfg.rumble_topic,
                    cfg.rate_hz
                );
                if !ff {
                    log::warn!(
                        "force feedback not supported on this pad/OS (macOS gilrs has no rumble); input still works"
                    );
                }
            }
            Err(e) => {
                log::warn!("{e:#}; will wait for a matching pad to connect");
            }
        }
    }

    let publisher = Arc::new(SharedPub {
        inner: Mutex::new(
            node.create_publisher::<XboxJoy>(&cfg.output_topic)
                .context("create XboxJoy publisher")?,
        ),
    });

    let (rumble_tx, rumble_rx) = mpsc::channel::<XboxJoyRumble>();
    let rumble_topic = cfg.rumble_topic.clone();
    node.create_subscription::<XboxJoyRumble, _>(
        &rumble_topic,
        move |_topic, msg| {
            if rumble_tx.send(msg).is_err() {
                log::debug!("rumble channel closed");
            }
        },
        None,
    )
    .context("create XboxJoyRumble subscription")?;

    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);
    let device = cfg.device.clone();
    let frame_id = cfg.frame_id.clone();
    let deadzone = cfg.deadzone;
    let period = Duration::from_secs_f64(1.0 / f64::from(cfg.rate_hz));
    let pub_thread = Arc::clone(&publisher);

    let join = thread::Builder::new()
        .name("xbox-joy".into())
        .spawn(move || {
            if let Err(e) = poll_loop(
                stop_thread,
                device,
                frame_id,
                deadzone,
                period,
                pub_thread,
                rumble_rx,
            ) {
                log::error!("xbox joy poll thread exited: {e:#}");
            }
        })
        .context("spawn xbox joy poll thread")?;

    let spin_result = node.spin().context("node spin");
    stop.store(true, Ordering::SeqCst);
    if let Err(e) = join.join() {
        log::warn!("xbox joy thread join: {e:?}");
    }
    spin_result
}

fn poll_loop(
    stop: Arc<AtomicBool>,
    device: String,
    frame_id: String,
    deadzone: f32,
    period: Duration,
    publisher: Arc<SharedPub>,
    rumble_rx: Receiver<XboxJoyRumble>,
) -> Result<()> {
    let mut gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("init gilrs in poll thread: {e}"))?;
    let mut active: Option<GamepadId> = resolve_joy_id(&gilrs, &device).ok();
    let mut rumble: Option<ActiveRumble> = None;
    let mut next_pub = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        while let Some(ev) = gilrs.next_event() {
            match ev.event {
                EventType::Connected => {
                    log::info!("joy connected: id={}", ev.id);
                    if active.is_none() {
                        active = resolve_joy_id(&gilrs, &device).ok();
                        if let Some(id) = active {
                            log::info!(
                                "using joy id={id} name={:?}",
                                gilrs.gamepad(id).name()
                            );
                        }
                    }
                }
                EventType::Disconnected => {
                    log::info!("joy disconnected: id={}", ev.id);
                    if active == Some(ev.id) {
                        rumble = None;
                        active = None;
                    }
                }
                _ => {}
            }
        }

        if active.is_none() {
            active = resolve_joy_id(&gilrs, &device).ok();
        }

        loop {
            match rumble_rx.try_recv() {
                Ok(cmd) => {
                    rumble = None;
                    if let Some(id) = active {
                        match apply_rumble(&mut gilrs, id, &cmd) {
                            Ok(next) => rumble = next,
                            Err(e) => log::warn!("apply rumble failed: {e:#}"),
                        }
                    } else {
                        log::warn!("rumble ignored: no active joy");
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }

        let now = Instant::now();
        if now >= next_pub {
            next_pub = now + period;
            if let Some(id) = active {
                if gilrs.gamepad(id).is_connected() {
                    let msg = to_xbox_joy(&gilrs.gamepad(id), &frame_id, deadzone);
                    if let Err(e) = publish_state(&publisher, &msg) {
                        log::warn!("publish XboxJoy failed: {e:#}");
                    }
                } else {
                    rumble = None;
                    active = None;
                }
            }
        }

        thread::sleep(Duration::from_millis(1));
    }

    drop(rumble);
    Ok(())
}

fn publish_state(publisher: &SharedPub, msg: &XboxJoy) -> Result<()> {
    let pub_ = publisher
        .inner
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    pub_.publish(msg).context("publish XboxJoy")?;
    Ok(())
}
