//! In-process differential-drive tank simulation (`tank`).
//!
//! Subscribes [`CMD_VEL_TOPIC`], integrates pose on an 11×11 world, and publishes
//! [`POSE_TOPIC`] at 20 Hz. Also serves:
//! - action [`POINT_NAV_ACTION`] — drive to one planar pose
//! - action [`MULTI_WAYPOINT_NAV_ACTION`] — visit poses in order
//! - service [`RESET_SERVICE`] — snap pose back to world center (home / 原点)
//!
//! Intended to run as a managed singleton beside the broker (console sessions
//! acquire/release it); multiple viewers share one world and `cmd_vel` is
//! last-writer-wins (ignored while an action is navigating).

use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::action::v1::{
    MultiWaypointNavigation, MultiWaypointNavigationFeedback, MultiWaypointNavigationGoal,
    MultiWaypointNavigationResult, PointNavigation, PointNavigationFeedback, PointNavigationGoal,
    PointNavigationResult,
};
use crate::geometry_msgs::msg::v1::{Pose2D, Twist};
use crate::robot_bus_interface::srv::v1::{Reset, ResetRequest, ResetResponse};
use crate::runtime::{CallbackGroupType, Context, MultiThreadedExecutor, Node, NodeOptions};
use crate::{ActionOutcome, BusError, Result};

/// Built-in tank demo namespace under the reserved `/robot_bus/*` prefix.
pub const TANK_PREFIX: &str = "/robot_bus/tank";
pub const CMD_VEL_TOPIC: &str = "/robot_bus/tank/cmd_vel";
pub const POSE_TOPIC: &str = "/robot_bus/tank/pose";
pub const POINT_NAV_ACTION: &str = "/robot_bus/tank/point_navigation";
pub const MULTI_WAYPOINT_NAV_ACTION: &str = "/robot_bus/tank/multi_waypoint_navigation";
pub const RESET_SERVICE: &str = "/robot_bus/tank/reset";
pub const WORLD_SIZE: f64 = 11.0;

const TICK: Duration = Duration::from_millis(50);
/// Deadman only: if cmd_vel stops arriving (client crash / drop), coast this long then halt.
/// Normal teleop publishes an explicit zero on key-up — do not rely on this for stop feel.
const CMD_TIMEOUT: Duration = Duration::from_millis(100);
const NAV_LINEAR: f64 = 1.5;
const NAV_ANGULAR: f64 = 2.2;
const POS_TOL: f64 = 0.08;
const YAW_TOL: f64 = 0.08;
/// Session lease — frontend should heartbeat more often than this.
pub const DEFAULT_LEASE: Duration = Duration::from_secs(15);
/// Delay before stopping the sim after the last session ends.
pub const DEFAULT_STOP_GRACE: Duration = Duration::from_secs(2);

/// Connect endpoints for the message / service / action buses (client-side).
#[derive(Clone, Debug)]
pub struct TankEndpoints {
    pub message_xsub: String,
    pub message_xpub: String,
    pub service_backend: String,
    pub action_backend: String,
}

struct SimState {
    x: f64,
    y: f64,
    theta: f64,
    linear: f64,
    angular: f64,
    last_cmd: Instant,
    /// True while an action owns the pose (teleop cmd_vel ignored).
    navigating: bool,
    /// Bumped by reset / newer goals so in-flight nav aborts.
    abort_token: u64,
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
            navigating: false,
            abort_token: 0,
        }
    }
}

impl SimState {
    fn pose(&self) -> Pose2D {
        Pose2D {
            x: self.x,
            y: self.y,
            theta: self.theta,
        }
    }

    fn snap_home(&mut self) {
        self.abort_token = self.abort_token.wrapping_add(1);
        self.navigating = false;
        self.x = WORLD_SIZE / 2.0;
        self.y = WORLD_SIZE / 2.0;
        self.theta = 0.0;
        self.linear = 0.0;
        self.angular = 0.0;
        self.last_cmd = Instant::now();
    }
}

/// Background physics node handle.
pub struct TankHandle {
    stop: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl TankHandle {
    /// Spawn `tank` on a dedicated thread (publisher stays thread-local).
    pub fn start(endpoints: TankEndpoints) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let ready = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let ready_flag = Arc::clone(&ready);
        let join = thread::Builder::new()
            .name("robot-bus-tank".into())
            .spawn(move || {
                if let Err(err) = run_loop(endpoints, stop_flag, ready_flag) {
                    eprintln!("tank exited: {err}");
                    log::error!("tank exited: {err}");
                }
            })
            .map_err(|e| BusError::Protocol(format!("spawn tank: {e}")))?;
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

impl Drop for TankHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

fn run_loop(
    endpoints: TankEndpoints,
    stop: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
) -> Result<()> {
    let mut opts = NodeOptions::tcp();
    opts.message_xsub = Some(endpoints.message_xsub);
    opts.message_xpub = Some(endpoints.message_xpub);
    opts.service_backend = Some(endpoints.service_backend);
    opts.action_backend = Some(endpoints.action_backend);

    let context = Context::new();
    // Worker pool so long-running nav actions don't block the pose tick.
    let executor = MultiThreadedExecutor::with_context(context.clone(), 4);
    let mut node = Node::with_context_options(&context, "tank", opts);
    executor.add_node(&mut node)?;

    let pose_pub = node.create_publisher::<Pose2D>(POSE_TOPIC)?;
    let state = Arc::new(Mutex::new(SimState::default()));

    {
        let state = Arc::clone(&state);
        node.create_subscription::<Twist, _>(
            CMD_VEL_TOPIC,
            move |_topic, twist| {
                let mut s = state.lock().expect("tank state");
                if s.navigating {
                    return;
                }
                s.linear = twist.linear.as_ref().map(|v| v.x).unwrap_or(0.0);
                s.angular = twist.angular.as_ref().map(|v| v.z).unwrap_or(0.0);
                s.last_cmd = Instant::now();
            },
            None,
        )?;
    }

    let rpc_group = node.create_callback_group(CallbackGroupType::Reentrant);

    {
        let state = Arc::clone(&state);
        node.create_service::<Reset, _>(
            RESET_SERVICE,
            move |_req: ResetRequest| {
                let mut s = state.lock().expect("tank state");
                s.snap_home();
                ResetResponse {
                    success: true,
                    msg: String::new(),
                }
            },
            Some(&rpc_group),
        )?;
    }

    {
        let state = Arc::clone(&state);
        node.create_action_server::<PointNavigation, _>(
            POINT_NAV_ACTION,
            move |goal: PointNavigationGoal| {
                let Some(pose) = goal.pose else {
                    abort_navigation(&state);
                    return ActionOutcome {
                        feedbacks: vec![],
                        result: PointNavigationResult {
                            success: false,
                            msg: "missing pose".into(),
                        },
                    };
                };
                let (ok, msg, feedbacks) =
                    navigate_waypoints(&state, &[(pose.x, pose.y, pose.theta)]);
                ActionOutcome {
                    feedbacks: feedbacks
                        .into_iter()
                        .map(|(current_pose, progress)| PointNavigationFeedback {
                            current_pose: Some(current_pose),
                            progress,
                        })
                        .collect(),
                    result: PointNavigationResult {
                        success: ok,
                        msg,
                    },
                }
            },
            Some(&rpc_group),
        )?;
    }

    {
        let state = Arc::clone(&state);
        node.create_action_server::<MultiWaypointNavigation, _>(
            MULTI_WAYPOINT_NAV_ACTION,
            move |goal: MultiWaypointNavigationGoal| {
                if goal.poses.is_empty() {
                    // Empty goal is used by the console as a soft cancel.
                    abort_navigation(&state);
                    return ActionOutcome {
                        feedbacks: vec![],
                        result: MultiWaypointNavigationResult {
                            success: false,
                            msg: "cancelled".into(),
                        },
                    };
                }
                let waypoints: Vec<(f64, f64, f64)> =
                    goal.poses.iter().map(|p| (p.x, p.y, p.theta)).collect();
                let (ok, msg, feedbacks) = navigate_waypoints(&state, &waypoints);
                ActionOutcome {
                    feedbacks: feedbacks
                        .into_iter()
                        .map(|(current_pose, progress)| MultiWaypointNavigationFeedback {
                            current_pose: Some(current_pose),
                            progress,
                        })
                        .collect(),
                    result: MultiWaypointNavigationResult {
                        success: ok,
                        msg,
                    },
                }
            },
            Some(&rpc_group),
        )?;
    }

    eprintln!(
        "tank online — SUB {CMD_VEL_TOPIC} → PUB {POSE_TOPIC}; \
         actions {POINT_NAV_ACTION}, {MULTI_WAYPOINT_NAV_ACTION}; \
         service {RESET_SERVICE} (tick {}ms)",
        TICK.as_millis()
    );
    log::info!(
        "tank online — SUB {CMD_VEL_TOPIC} → PUB {POSE_TOPIC}; \
         actions {POINT_NAV_ACTION}, {MULTI_WAYPOINT_NAV_ACTION}; \
         service {RESET_SERVICE} (tick {}ms)",
        TICK.as_millis()
    );

    let mut last_tick = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        executor.spin_once(Some(Duration::from_millis(5)))?;

        let now = Instant::now();
        if now.duration_since(last_tick) < TICK {
            continue;
        }
        let dt = (now - last_tick).as_secs_f64().min(0.05);
        last_tick = now;

        let pose = {
            let mut s = state.lock().expect("tank state");
            if !s.navigating {
                if now.duration_since(s.last_cmd) > CMD_TIMEOUT {
                    s.linear = 0.0;
                    s.angular = 0.0;
                }
                s.theta += s.angular * dt;
                s.x = (s.x + s.theta.cos() * s.linear * dt).clamp(0.0, WORLD_SIZE);
                s.y = (s.y + s.theta.sin() * s.linear * dt).clamp(0.0, WORLD_SIZE);
            }
            s.pose()
        };

        if let Err(err) = pose_pub.publish(&pose) {
            log::warn!("tank: publish {POSE_TOPIC} failed: {err}");
        } else {
            ready.store(true, Ordering::Relaxed);
        }
    }

    let _ = node.shutdown();
    log::info!("tank stopped");
    Ok(())
}

fn abort_navigation(state: &Arc<Mutex<SimState>>) {
    let mut s = state.lock().expect("tank state");
    s.abort_token = s.abort_token.wrapping_add(1);
    s.navigating = false;
    s.linear = 0.0;
    s.angular = 0.0;
}

/// Drive through planar waypoints; returns (success, msg, feedback samples).
fn navigate_waypoints(
    state: &Arc<Mutex<SimState>>,
    waypoints: &[(f64, f64, f64)],
) -> (bool, String, Vec<(Pose2D, f32)>) {
    let token = {
        let mut s = state.lock().expect("tank state");
        s.abort_token = s.abort_token.wrapping_add(1);
        let token = s.abort_token;
        s.navigating = true;
        s.linear = 0.0;
        s.angular = 0.0;
        token
    };

    let mut feedbacks = Vec::new();
    let mut ok = true;
    let mut msg = String::new();
    let n = waypoints.len().max(1) as f32;

    for (i, &(gx, gy, gtheta)) in waypoints.iter().enumerate() {
        let gx = gx.clamp(0.0, WORLD_SIZE);
        let gy = gy.clamp(0.0, WORLD_SIZE);
        let match_yaw = i + 1 == waypoints.len();
        match drive_to_pose(state, token, gx, gy, gtheta, match_yaw, |pose, local| {
            let overall = (i as f32 + local) / n;
            feedbacks.push((pose, overall.clamp(0.0, 1.0)));
        }) {
            Ok(()) => {}
            Err(reason) => {
                ok = false;
                msg = reason;
                break;
            }
        }
    }

    {
        let mut s = state.lock().expect("tank state");
        if s.abort_token == token {
            s.navigating = false;
            s.linear = 0.0;
            s.angular = 0.0;
        }
    }

    if ok {
        feedbacks.push({
            let s = state.lock().expect("tank state");
            (s.pose(), 1.0)
        });
    }
    (ok, msg, feedbacks)
}

fn drive_to_pose(
    state: &Arc<Mutex<SimState>>,
    token: u64,
    gx: f64,
    gy: f64,
    gtheta: f64,
    match_yaw: bool,
    mut on_progress: impl FnMut(Pose2D, f32),
) -> std::result::Result<(), String> {
    let start = {
        let s = state.lock().expect("tank state");
        if s.abort_token != token {
            return Err("aborted".into());
        }
        (s.x, s.y)
    };
    let path_len = ((gx - start.0).hypot(gy - start.1)).max(1e-3);

    // 1) Face the goal, 2) drive, 3) optionally match yaw (final waypoint only).
    let mut step: u32 = 0;
    loop {
        let (pose, phase_done, local_progress) = {
            let mut s = state.lock().expect("tank state");
            if s.abort_token != token {
                return Err("aborted".into());
            }
            let dx = gx - s.x;
            let dy = gy - s.y;
            let dist = dx.hypot(dy);
            let bearing = dy.atan2(dx);
            let traveled = (1.0 - dist / path_len).clamp(0.0, 1.0) as f32;

            if dist > POS_TOL {
                let yaw_err = angle_diff(s.theta, bearing);
                if yaw_err.abs() > YAW_TOL {
                    let step_ang = NAV_ANGULAR * TICK.as_secs_f64();
                    s.theta += yaw_err.signum() * step_ang.min(yaw_err.abs());
                    (s.pose(), false, traveled * 0.85)
                } else {
                    let step_lin = NAV_LINEAR * TICK.as_secs_f64();
                    let move_by = step_lin.min(dist);
                    s.x = (s.x + s.theta.cos() * move_by).clamp(0.0, WORLD_SIZE);
                    s.y = (s.y + s.theta.sin() * move_by).clamp(0.0, WORLD_SIZE);
                    (s.pose(), false, traveled * 0.85)
                }
            } else if match_yaw {
                let yaw_err = angle_diff(s.theta, gtheta);
                if yaw_err.abs() > YAW_TOL {
                    let step_ang = NAV_ANGULAR * TICK.as_secs_f64();
                    s.theta += yaw_err.signum() * step_ang.min(yaw_err.abs());
                    (
                        s.pose(),
                        false,
                        0.85 + (1.0 - (yaw_err.abs() / PI) as f32) * 0.15,
                    )
                } else {
                    s.x = gx;
                    s.y = gy;
                    s.theta = gtheta;
                    (s.pose(), true, 1.0)
                }
            } else {
                s.x = gx;
                s.y = gy;
                (s.pose(), true, 1.0)
            }
        };

        step = step.wrapping_add(1);
        if phase_done || step % 4 == 0 {
            on_progress(pose, local_progress);
        }
        if phase_done {
            return Ok(());
        }
        thread::sleep(TICK);
    }
}

fn angle_diff(from: f64, to: f64) -> f64 {
    let mut d = (to - from) % (2.0 * PI);
    if d > PI {
        d -= 2.0 * PI;
    } else if d < -PI {
        d += 2.0 * PI;
    }
    d
}

/// Result of creating a viewer/control session.
#[derive(Clone, Debug)]
pub struct TankSession {
    pub session_id: String,
    pub lease: Duration,
    pub viewers: usize,
}

/// Snapshot for status APIs.
#[derive(Clone, Debug)]
pub struct TankStatus {
    pub running: bool,
    pub viewers: usize,
}

struct ManagerInner {
    handle: Option<TankHandle>,
    sessions: HashMap<String, Instant>,
    stop_after: Option<Instant>,
}

/// Ref-counted session manager: first acquire starts sim, last release (+ grace) stops it.
pub struct TankManager {
    endpoints: TankEndpoints,
    lease: Duration,
    stop_grace: Duration,
    inner: Mutex<ManagerInner>,
}

impl TankManager {
    pub fn new(endpoints: TankEndpoints) -> Arc<Self> {
        Self::with_timing(endpoints, DEFAULT_LEASE, DEFAULT_STOP_GRACE)
    }

    pub fn with_timing(
        endpoints: TankEndpoints,
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

    pub fn acquire(&self) -> Result<TankSession> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        self.sweep_locked(&mut inner)?;

        if inner.handle.is_none() {
            let handle = TankHandle::start(self.endpoints.clone())?;
            inner.handle = Some(handle);
            inner.stop_after = None;
        }

        // Wait for first pose outside the lock so other acquires can proceed.
        let ready_handle = inner.handle.as_ref().map(|h| Arc::clone(&h.ready));
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

        Ok(TankSession {
            session_id,
            lease: self.lease,
            viewers,
        })
    }

    pub fn heartbeat(&self, session_id: &str) -> Result<TankSession> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        self.sweep_locked(&mut inner)?;
        match inner.sessions.get_mut(session_id) {
            Some(last) => {
                *last = Instant::now();
                Ok(TankSession {
                    session_id: session_id.to_string(),
                    lease: self.lease,
                    viewers: inner.sessions.len(),
                })
            }
            None => Err(BusError::Protocol(format!(
                "tank session not found: {session_id}"
            ))),
        }
    }

    pub fn release(&self, session_id: &str) -> Result<TankStatus> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.sessions.remove(session_id);
        if inner.sessions.is_empty() {
            inner.stop_after = Some(Instant::now() + self.stop_grace);
        }
        self.sweep_locked(&mut inner)?;
        Ok(TankStatus {
            running: inner.handle.is_some(),
            viewers: inner.sessions.len(),
        })
    }

    pub fn status(&self) -> TankStatus {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let _ = self.sweep_locked(&mut inner);
        TankStatus {
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
        let mgr = TankManager::with_timing(
            TankEndpoints {
                message_xsub: "tcp://127.0.0.1:1".into(),
                message_xpub: "tcp://127.0.0.1:1".into(),
                service_backend: "tcp://127.0.0.1:1".into(),
                action_backend: "tcp://127.0.0.1:1".into(),
            },
            Duration::from_millis(200),
            Duration::from_millis(50),
        );

        let st = mgr.status();
        assert!(!st.running);
        assert_eq!(st.viewers, 0);
    }
}
