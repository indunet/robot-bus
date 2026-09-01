//! Typed mapper for `control_msgs/msg/MotionPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn motion_primitive_to_bus(msg: ros_env::control_msgs::msg::MotionPrimitive) -> crate::control_msgs::msg::v1::MotionPrimitive {
    crate::control_msgs::msg::v1::MotionPrimitive {
        r#type: msg.type_,
        blend_radius: msg.blend_radius,
        additional_arguments: msg.additional_arguments.into_iter().map(crate::ros2_bridge::mappers::control_msgs::motion_argument::motion_argument_to_bus).collect(),
        poses: msg.poses.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_bus).collect(),
        joint_positions: crate::ros2_bridge::mappers::convert::f64_seq(msg.joint_positions),
    }
}

pub(crate) fn motion_primitive_to_ros(bus: crate::control_msgs::msg::v1::MotionPrimitive) -> ros_env::control_msgs::msg::MotionPrimitive {
    ros_env::control_msgs::msg::MotionPrimitive {
        type_: bus.r#type,
        blend_radius: bus.blend_radius,
        additional_arguments: bus.additional_arguments.into_iter().map(crate::ros2_bridge::mappers::control_msgs::motion_argument::motion_argument_to_ros).collect(),
        poses: bus.poses.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_ros).collect(),
        joint_positions: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.joint_positions),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsMotionPrimitiveMapper;

impl TypedTopicMapper for ControlMsgsMotionPrimitiveMapper {
    type Ros = ros_env::control_msgs::msg::MotionPrimitive;
    type Bus = crate::control_msgs::msg::v1::MotionPrimitive;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(motion_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(motion_primitive_to_ros(msg))
    }
}
