//! Build `sensor_msgs/Image` (`rgb8`) from decoded RGB pixels.

use crate::builtin_interfaces::msg::v1::Time;
use crate::sensor_msgs::msg::v1::Image;
use crate::std_msgs::msg::v1::Header;
use std::time::{SystemTime, UNIX_EPOCH};

/// Pack an RGB8 buffer into a ROS-style Image message.
pub fn rgb8_image(width: u32, height: u32, frame_id: &str, data: Vec<u8>) -> Image {
    let step = width.saturating_mul(3);
    Image {
        header: Some(Header {
            stamp: Some(now_time()),
            frame_id: frame_id.to_string(),
        }),
        height,
        width,
        encoding: "rgb8".into(),
        is_bigendian: false,
        step,
        data,
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
    fn rgb8_step_and_encoding() {
        let img = rgb8_image(2, 1, "cam", vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(img.encoding, "rgb8");
        assert_eq!(img.step, 6);
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        assert_eq!(img.header.as_ref().unwrap().frame_id, "cam");
    }
}
