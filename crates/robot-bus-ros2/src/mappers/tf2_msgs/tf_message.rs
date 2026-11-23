//! Typed mapper for `tf2_msgs/msg/TFMessage`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn tf_message_to_bus(
    msg: ros_env::tf2_msgs::msg::TFMessage,
) -> robot_bus::tf2_msgs::msg::v1::TfMessage {
    robot_bus::tf2_msgs::msg::v1::TfMessage {
        transforms: msg.transforms.into_iter().map(crate::mappers::geometry_msgs::transform_stamped::transform_stamped_to_bus).collect(),
    }
}

pub(crate) fn tf_message_to_ros(
    bus: robot_bus::tf2_msgs::msg::v1::TfMessage,
) -> ros_env::tf2_msgs::msg::TFMessage {
    ros_env::tf2_msgs::msg::TFMessage {
        transforms: bus.transforms.into_iter().map(crate::mappers::geometry_msgs::transform_stamped::transform_stamped_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Tf2MsgsTfMessageMapper;

impl TypedTopicMapper for Tf2MsgsTfMessageMapper {
    type Ros = ros_env::tf2_msgs::msg::TFMessage;
    type Bus = robot_bus::tf2_msgs::msg::v1::TfMessage;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(tf_message_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(tf_message_to_ros(msg))
    }
}
