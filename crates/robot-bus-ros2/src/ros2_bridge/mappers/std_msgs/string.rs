//! Typed mapper for `std_msgs/msg/String`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn string_to_bus(
    msg: ros_env::std_msgs::msg::String,
) -> crate::std_msgs::msg::v1::String {
    crate::std_msgs::msg::v1::String {
        data: crate::ros2_bridge::mappers::convert::from_ros_string(msg.data),
    }
}

pub(crate) fn string_to_ros(
    bus: crate::std_msgs::msg::v1::String,
) -> ros_env::std_msgs::msg::String {
    ros_env::std_msgs::msg::String {
        data: crate::ros2_bridge::mappers::convert::to_ros_string(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdMsgsStringMapper;

impl TypedTopicMapper for StdMsgsStringMapper {
    type Ros = ros_env::std_msgs::msg::String;
    type Bus = crate::std_msgs::msg::v1::String;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(string_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(string_to_ros(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_roundtrip() {
        let ros = ros_env::std_msgs::msg::String {
            data: "hello".into(),
        };
        let bus = string_to_bus(ros);
        assert_eq!(bus.data, "hello");
        let back = string_to_ros(bus);
        assert_eq!(back.data.to_string(), "hello");
    }
}
