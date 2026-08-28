//! Typed mapper for `sensor_msgs/msg/Image`.
//!
//! Owned `ros_to_bus` so bulk `data` moves (`IntoU8Vec`) instead of
//! `iter().copied().collect()`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn image_to_bus(
    msg: ros_env::sensor_msgs::msg::Image,
) -> crate::sensor_msgs::msg::v1::Image {
    crate::sensor_msgs::msg::v1::Image {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        height: msg.height,
        width: msg.width,
        encoding: crate::ros2_bridge::mappers::convert::from_ros_string(msg.encoding),
        is_bigendian: crate::ros2_bridge::mappers::convert::octet_to_bool(msg.is_bigendian),
        step: msg.step,
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn image_to_ros(
    bus: crate::sensor_msgs::msg::v1::Image,
) -> ros_env::sensor_msgs::msg::Image {
    ros_env::sensor_msgs::msg::Image {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        height: bus.height,
        width: bus.width,
        encoding: crate::ros2_bridge::mappers::convert::to_ros_string(bus.encoding),
        is_bigendian: crate::ros2_bridge::mappers::convert::bool_to_octet(bus.is_bigendian),
        step: bus.step,
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsImageMapper;

impl TypedTopicMapper for SensorMsgsImageMapper {
    type Ros = ros_env::sensor_msgs::msg::Image;
    type Bus = crate::sensor_msgs::msg::v1::Image;

    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Image"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(image_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(image_to_ros(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_moves_data() {
        let ros = ros_env::sensor_msgs::msg::Image {
            header: Default::default(),
            height: 1,
            width: 2,
            encoding: "rgb8".into(),
            is_bigendian: 0,
            step: 6,
            data: vec![1, 2, 3, 4, 5, 6],
        };
        let bus = image_to_bus(ros);
        assert_eq!(bus.height, 1);
        assert_eq!(bus.width, 2);
        assert!(!bus.is_bigendian);
        assert_eq!(bus.data, vec![1, 2, 3, 4, 5, 6]);
        let back = image_to_ros(bus);
        assert_eq!(back.data, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(back.is_bigendian, 0);
        assert_eq!(back.encoding.to_string(), "rgb8");
    }
}
