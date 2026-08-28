//! Typed mapper for `sensor_msgs/msg/PointCloud2`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_cloud2_to_bus(msg: ros_env::sensor_msgs::msg::PointCloud2) -> crate::sensor_msgs::msg::v1::PointCloud2 {
    crate::sensor_msgs::msg::v1::PointCloud2 {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        height: msg.height,
        width: msg.width,
        fields: msg.fields.into_iter().map(crate::ros2_bridge::mappers::sensor_msgs::point_field::point_field_to_bus).collect(),
        is_bigendian: msg.is_bigendian,
        point_step: msg.point_step,
        row_step: msg.row_step,
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
        is_dense: msg.is_dense,
    }
}

pub(crate) fn point_cloud2_to_ros(bus: crate::sensor_msgs::msg::v1::PointCloud2) -> ros_env::sensor_msgs::msg::PointCloud2 {
    ros_env::sensor_msgs::msg::PointCloud2 {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        height: bus.height,
        width: bus.width,
        fields: bus.fields.into_iter().map(crate::ros2_bridge::mappers::sensor_msgs::point_field::point_field_to_ros).collect(),
        is_bigendian: bus.is_bigendian,
        point_step: bus.point_step,
        row_step: bus.row_step,
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
        is_dense: bus.is_dense,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsPointCloud2Mapper;

impl TypedTopicMapper for SensorMsgsPointCloud2Mapper {
    type Ros = ros_env::sensor_msgs::msg::PointCloud2;
    type Bus = crate::sensor_msgs::msg::v1::PointCloud2;

    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/PointCloud2"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_cloud2_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_cloud2_to_ros(msg))
    }
}
