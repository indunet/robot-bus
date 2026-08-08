//! In-process differential-drive bot simulation (`bot_sim`).
//!
//! Subscribes [`CMD_VEL_TOPIC`], integrates pose on an 11×11 world, and publishes
//! [`POSE_TOPIC`] at 20 Hz. Intended to run as a managed singleton beside the
//! broker (console sessions acquire/release it); multiple viewers share one world
//! and `cmd_vel` is last-writer-wins.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::geometry_msgs::msg::v1::{Pose2D, Twist};
use crate::runtime::{Node, NodeOptions};
use crate::{BusError, Result};

pub const CMD_VEL_TOPIC: &str = "/bot1/cmd_vel";
pub const POSE_TOPIC: &str = "/bot1/pose";
pub const WORLD_SIZE: f64 = 11.0;

const TICK: Duration = Duration::from_millis(50);
const CMD_TIMEOUT: Duration = Duration::from_millis(400);
/// Session lease — frontend should heartbeat more often than this.
pub const DEFAULT_LEASE: Duration = Duration::from_secs(15);
/// Delay before stopping the sim after the last session ends.
pub const DEFAULT_STOP_GRACE: Duration = Duration::from_secs(2);

/// Connect endpoints for the message bus (client-side, not bind addresses).
#[derive(Clone, Debug)]
pub struct BotSimEndpoints {
    pub message_xsub: String,
    pub message_xpub: String,
}

struct SimState {
    x: f64,
    y: f64,
    theta: f64,
    linear: f64,
    angular: f64,
    last_cmd: Instant,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            x: WORLD_SIZE / 2.0,
            y: WORLD_SIZE / 2.0,
            theta: 0.0,
            linear: 0.0,
            angular: 0.0,
            last_cmd: Instant::now(),
        }
    }
}

/// Background physics node handle.
pub struct BotSimHandle {
    stop: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl BotSimHandle {
    /// Spawn `bot_sim` on a dedicated thread (publisher stays thread-local).
    pub fn start(endpoints: BotSimEndpoints) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let ready_flag = Arc::clone(&ready);
        let join = thread::Builder::new()
            .name("robot-bus-bot-sim".into())
            .spawn(move || {
                if let Err(err) = run_loop(endpoints, stop_flag, ready_flag) {
                    eprintln!("bot_sim exited: {err}");
                    log::error!("bot_sim exited: {err}");
                }
            })
            .map_err(|e| BusError::Protocol(format!("spawn bot_sim: {e}")))?;
        Ok(Self {
            stop,
            ready,
            join: Some(join),
        })
    }

    /// Wait until the sim loop has published at least once (or timeout).
    pub fn wait_ready(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.ready.load(Ordering::Relaxed) {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        self.ready.load(Ordering::Relaxed)
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn stop(mut self) {
        self.request_stop();
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BotSimHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

fn run_loop(
    endpoints: BotSimEndpoints,
    stop: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
) -> Result<()> {
    let mut opts = NodeOptions::tcp();
    opts.message_xsub = Some(endpoints.message_xsub);
    opts.message_xpub = Some(endpoints.message_xpub);
    let mut node = Node::with_options("bot_sim", opts);
    let pose_pub = node.create_publisher::<Pose2D>(POSE_TOPIC)?;
    let state = Arc::new(Mutex::new(SimState::default()));

    {
        let state = Arc::clone(&state);
        node.create_subscription::<Twist, _>(
            CMD_VEL_TOPIC,
            move |_topic, twist| {
                let mut s = state.lock().expect("bot_sim state");
                s.linear = twist.linear.as_ref().map(|v| v.x).unwrap_or(0.0);
                s.angular = twist.angular.as_ref().map(|v| v.z).unwrap_or(0.0);
                s.last_cmd = Instant::now();
            },
            None,
        )?;
    }

    eprintln!(
        "bot_sim online — SUB {CMD_VEL_TOPIC} → PUB {POSE_TOPIC} (tick {}ms)",
        TICK.as_millis()
    );
    log::info!(
        "bot_sim online — SUB {CMD_VEL_TOPIC} → PUB {POSE_TOPIC} (tick {}ms)",
        TICK.as_millis()
    );

    let mut last_tick = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        node.spin_once(Some(Duration::from_millis(5)))?;

        let now = Instant::now();
        if now.duration_since(last_tick) < TICK {
            continue;
        }
        let dt = (now - last_tick).as_secs_f64().min(0.05);
        last_tick = now;

        let pose = {
            let mut s = state.lock().expect("bot_sim state");
            if now.duration_since(s.last_cmd) > CMD_TIMEOUT {
                s.linear = 0.0;
                s.angular = 0.0;
            }

            s.theta += s.angular * dt;
            s.x = (s.x + s.theta.cos() * s.linear * dt).clamp(0.0, WORLD_SIZE);
            s.y = (s.y + s.theta.sin() * s.linear * dt).clamp(0.0, WORLD_SIZE);

            Pose2D {
                x: s.x,
                y: s.y,
                theta: s.theta,
            }
        };

        if let Err(err) = pose_pub.publish(&pose) {
            log::warn!("bot_sim: publish {POSE_TOPIC} failed: {err}");
        } else {
            ready.store(true, Ordering::Relaxed);
        }
    }

    let _ = node.shutdown();
    log::info!("bot_sim stopped");
    Ok(())
}

/// Result of creating a viewer/control session.
#[derive(Clone, Debug)]
pub struct BotSimSession {
    pub session_id: String,
    pub lease: Duration,
    pub viewers: usize,
}

/// Snapshot for status APIs.
#[derive(Clone, Debug)]
pub struct BotSimStatus {
    pub running: bool,
    pub viewers: usize,
}

struct ManagerInner {
    handle: Option<BotSimHandle>,
    sessions: HashMap<String, Instant>,
    stop_after: Option<Instant>,
}

/// Ref-counted session manager: first acquire starts sim, last release (+ grace) stops it.
pub struct BotSimManager {
    endpoints: BotSimEndpoints,
    lease: Duration,
    stop_grace: Duration,
    inner: Mutex<ManagerInner>,
}

impl BotSimManager {
    pub fn new(endpoints: BotSimEndpoints) -> Arc<Self> {
        Self::with_timing(endpoints, DEFAULT_LEASE, DEFAULT_STOP_GRACE)
    }

    pub fn with_timing(
        endpoints: BotSimEndpoints,
        lease: Duration,
        stop_grace: Duration,
    ) -> Arc<Self> {
        Arc::new(Self {
            endpoints,
            lease,
            stop_grace,
            inner: Mutex::new(ManagerInner {
                handle: None,
                sessions: HashMap::new(),
                stop_after: None,
            }),
        })
    }

    pub fn lease(&self) -> Duration {
        self.lease
    }

    pub fn acquire(&self) -> Result<BotSimSession> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        self.sweep_locked(&mut inner)?;

        if inner.handle.is_none() {
            let handle = BotSimHandle::start(self.endpoints.clone())?;
            inner.handle = Some(handle);
            inner.stop_after = None;
        }

        // Wait for first pose outside the lock so other acquires can proceed.
        let ready_handle = inner
            .handle
            .as_ref()
            .map(|h| Arc::clone(&h.ready));
        let session_id = Uuid::new_v4().to_string();
        inner.sessions.insert(session_id.clone(), Instant::now());
        let viewers = inner.sessions.len();
        drop(inner);

        if let Some(ready) = ready_handle {
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                if ready.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        }

        Ok(BotSimSession {
            session_id,
            lease: self.lease,
            viewers,
        })
    }

    pub fn heartbeat(&self, session_id: &str) -> Result<BotSimSession> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        self.sweep_locked(&mut inner)?;
        match inner.sessions.get_mut(session_id) {
            Some(last) => {
                *last = Instant::now();
                Ok(BotSimSession {
                    session_id: session_id.to_string(),
                    lease: self.lease,
                    viewers: inner.sessions.len(),
                })
            }
            None => Err(BusError::Protocol(format!(
                "bot_sim session not found: {session_id}"
            ))),
        }
    }

    pub fn release(&self, session_id: &str) -> Result<BotSimStatus> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.sessions.remove(session_id);
        if inner.sessions.is_empty() {
            inner.stop_after = Some(Instant::now() + self.stop_grace);
        }
        self.sweep_locked(&mut inner)?;
        Ok(BotSimStatus {
            running: inner.handle.is_some(),
            viewers: inner.sessions.len(),
        })
    }

    pub fn status(&self) -> BotSimStatus {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let _ = self.sweep_locked(&mut inner);
        BotSimStatus {
            running: inner.handle.is_some(),
            viewers: inner.sessions.len(),
        }
    }

    /// Force-stop the sim and drop all sessions (broker shutdown).
    pub fn shutdown(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.sessions.clear();
        inner.stop_after = None;
        if let Some(handle) = inner.handle.take() {
            handle.stop();
        }
    }

    fn sweep_locked(&self, inner: &mut ManagerInner) -> Result<()> {
        let now = Instant::now();
        inner
            .sessions
            .retain(|_, last| now.duration_since(*last) <= self.lease);

        if inner.sessions.is_empty() {
            let should_stop = match inner.stop_after {
                Some(deadline) => now >= deadline,
                None => {
                    // Lease expiry emptied sessions without an explicit release — stop after grace.
                    if inner.handle.is_some() {
                        inner.stop_after = Some(now + self.stop_grace);
                    }
                    false
                }
            };
            if should_stop && let Some(handle) = inner.handle.take() {
                handle.stop();
                inner.stop_after = None;
            }
        } else {
            inner.stop_after = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_tracks_sessions_without_bus() {
        // acquire starts a real Node — skip if no broker. Unit-test sweep math only.
        let mgr = BotSimManager::with_timing(
            BotSimEndpoints {
                message_xsub: "tcp://127.0.0.1:1".into(),
                message_xpub: "tcp://127.0.0.1:1".into(),
            },
            Duration::from_millis(200),
            Duration::from_millis(50),
        );

        // Inject a fake running state via status after failed start is awkward;
        // exercise release/status on empty manager.
        let st = mgr.status();
        assert!(!st.running);
        assert_eq!(st.viewers, 0);
    }
}
