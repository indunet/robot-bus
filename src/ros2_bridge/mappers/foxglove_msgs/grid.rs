//! Typed mapper for `foxglove_msgs/msg/Grid`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn grid_to_bus(msg: ros_env::foxglove_msgs::msg::Grid) -> crate::foxglove_msgs::msg::v1::Grid {
    crate::foxglove_msgs::msg::v1::Grid {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        column_count: msg.column_count,
        cell_size: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector2::vector2_to_bus(msg.cell_size)),
        row_stride: msg.row_stride,
        cell_stride: msg.cell_stride,
        fields: msg.fields.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::packed_element_field::packed_element_field_to_bus).collect(),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn grid_to_ros(bus: crate::foxglove_msgs::msg::v1::Grid) -> ros_env::foxglove_msgs::msg::Grid {
    ros_env::foxglove_msgs::msg::Grid {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        column_count: bus.column_count,
        cell_size: crate::ros2_bridge::mappers::foxglove_msgs::vector2::vector2_to_ros(bus.cell_size.unwrap_or_default()),
        row_stride: bus.row_stride,
        cell_stride: bus.cell_stride,
        fields: bus.fields.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::packed_element_field::packed_element_field_to_ros).collect(),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsGridMapper;

impl TypedTopicMapper for FoxgloveMsgsGridMapper {
    type Ros = ros_env::foxglove_msgs::msg::Grid;
    type Bus = crate::foxglove_msgs::msg::v1::Grid;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(grid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(grid_to_ros(msg))
    }
}
