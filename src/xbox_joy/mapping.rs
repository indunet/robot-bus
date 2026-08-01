//! Map gilrs pad state → `XboxJoy`.

use crate::builtin_interfaces::msg::v1::Time;
use crate::robot_bus_interface::msg::v1::XboxJoy;
use crate::std_msgs::msg::v1::Header;
use gilrs::{Axis, Button, Gamepad};
use std::time::{SystemTime, UNIX_EPOCH};

/// Snapshot current pad state into an Xbox-layout joy message.
pub fn to_xbox_joy(pad: &Gamepad<'_>, frame_id: &str, deadzone: f32) -> XboxJoy {
    XboxJoy {
        header: Some(Header {
            stamp: Some(now_time()),
            frame_id: frame_id.to_string(),
        }),
        left_stick_x: apply_deadzone(pad.value(Axis::LeftStickX), deadzone),
        left_stick_y: apply_deadzone(pad.value(Axis::LeftStickY), deadzone),
        right_stick_x: apply_deadzone(pad.value(Axis::RightStickX), deadzone),
        right_stick_y: apply_deadzone(pad.value(Axis::RightStickY), deadzone),
        left_trigger: trigger_01(pad.value(Axis::LeftZ)),
        right_trigger: trigger_01(pad.value(Axis::RightZ)),
        // gilrs unified layout → Xbox face positions.
        a: pad.is_pressed(Button::South),
        b: pad.is_pressed(Button::East),
        x: pad.is_pressed(Button::West),
        y: pad.is_pressed(Button::North),
        left_bumper: pad.is_pressed(Button::LeftTrigger),
        right_bumper: pad.is_pressed(Button::RightTrigger),
        left_stick_button: pad.is_pressed(Button::LeftThumb),
        right_stick_button: pad.is_pressed(Button::RightThumb),
        dpad_up: pad.is_pressed(Button::DPadUp),
        dpad_down: pad.is_pressed(Button::DPadDown),
        dpad_left: pad.is_pressed(Button::DPadLeft),
        dpad_right: pad.is_pressed(Button::DPadRight),
        view: pad.is_pressed(Button::Select),
        menu: pad.is_pressed(Button::Start),
        guide: pad.is_pressed(Button::Mode),
    }
}

fn apply_deadzone(v: f32, deadzone: f32) -> f32 {
    if v.abs() < deadzone {
        0.0
    } else {
        v.clamp(-1.0, 1.0)
    }
}

/// Normalize trigger axis to 0..1 (handles both 0..1 and −1..1 driver ranges).
fn trigger_01(v: f32) -> f32 {
    if v < 0.0 {
        ((v + 1.0) * 0.5).clamp(0.0, 1.0)
    } else {
        v.clamp(0.0, 1.0)
    }
}

fn now_time() -> Time {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Time {
        sec: dur.as_secs() as i32,
        nanosec: dur.subsec_nanos(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadzone_zeros_small() {
        assert_eq!(apply_deadzone(0.05, 0.1), 0.0);
        assert_eq!(apply_deadzone(-0.05, 0.1), 0.0);
        assert!((apply_deadzone(0.5, 0.1) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn trigger_maps_signed() {
        assert!((trigger_01(-1.0) - 0.0).abs() < f32::EPSILON);
        assert!((trigger_01(1.0) - 1.0).abs() < f32::EPSILON);
        assert!((trigger_01(0.5) - 0.5).abs() < f32::EPSILON);
    }
}
