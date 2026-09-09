//! Env knobs, QoS, and payload helpers for `ros2_bridge_perf`.

use std::time::Duration;

use crate::support::{env_f64, env_usize};
use robot_bus::QosProfile;
use ros_env::sensor_msgs::msg::Image as RosImage;

pub(crate) const STRING_LEN: usize = 64;
pub(crate) const MSG_HWM: i32 = 2_048;
pub(crate) const WARMUP: usize = 20;

pub(crate) fn image_width() -> u32 {
    env_usize("ROS2_BRIDGE_PERF_IMAGE_WIDTH", 640) as u32
}

pub(crate) fn image_height() -> u32 {
    env_usize("ROS2_BRIDGE_PERF_IMAGE_HEIGHT", 480) as u32
}

pub(crate) fn max_loss_pct() -> f64 {
    env_f64("ROS2_BRIDGE_PERF_MAX_LOSS_PCT", 1.0)
}

pub(crate) fn goodput_trial_secs() -> f64 {
    env_f64("ROS2_BRIDGE_PERF_GOODPUT_TRIAL_SECS", 1.0)
}

pub(crate) fn goodput_rate_lo() -> u64 {
    env_usize("ROS2_BRIDGE_PERF_GOODPUT_RATE_LO", 50) as u64
}

pub(crate) fn goodput_rate_hi() -> u64 {
    env_usize("ROS2_BRIDGE_PERF_GOODPUT_RATE_HI", 50_000) as u64
}

pub(crate) fn msg_latency_samples() -> usize {
    env_usize("ROS2_BRIDGE_PERF_MSG_LATENCY_SAMPLES", 200)
}

pub(crate) fn goodput_settle() -> Duration {
    Duration::from_millis(env_usize("ROS2_BRIDGE_PERF_GOODPUT_SETTLE_MS", 100) as u64)
}

pub(crate) fn string_payload(ts_ns: u64) -> String {
    format!("{ts_ns:016x}{}", "x".repeat(STRING_LEN.saturating_sub(16)))
}

pub(crate) fn parse_ts(data: &str) -> Option<u64> {
    u64::from_str_radix(data.get(..16)?, 16).ok()
}

pub(crate) fn make_image(ts_ns: u64) -> RosImage {
    let w = image_width();
    let h = image_height();
    let mut data = vec![0u8; (w * h * 3) as usize];
    data[..8].copy_from_slice(&ts_ns.to_le_bytes());
    RosImage {
        header: Default::default(),
        height: h,
        width: w,
        encoding: "rgb8".into(),
        is_bigendian: 0,
        step: w * 3,
        data,
    }
}

pub(crate) fn image_ts(data: &[u8]) -> Option<u64> {
    if data.len() < 8 {
        return None;
    }
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[..8]);
    Some(u64::from_le_bytes(b))
}

pub(crate) fn rpc_qos() -> QosProfile {
    QosProfile::keep_last(MSG_HWM)
}
