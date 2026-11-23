//! Typed mapper for `sensor_msgs/msg/PointCloud2`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn point_cloud2_to_bus(
    msg: ros_env::sensor_msgs::msg::PointCloud2,
) -> robot_bus::sensor_msgs::msg::v1::PointCloud2 {
    robot_bus::sensor_msgs::msg::v1::PointCloud2 {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        height: msg.height.into(),
        width: msg.width.into(),
        fields: msg
            .fields
            .into_iter()
            .map(crate::mappers::sensor_msgs::point_field::point_field_to_bus)
            .collect(),
        is_bigendian: msg.is_bigendian,
        point_step: msg.point_step.into(),
        row_step: msg.row_step.into(),
        data: crate::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
        is_dense: msg.is_dense,
    }
}

pub(crate) fn point_cloud2_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::PointCloud2,
) -> ros_env::sensor_msgs::msg::PointCloud2 {
    ros_env::sensor_msgs::msg::PointCloud2 {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        height: bus.height as _,
        width: bus.width as _,
        fields: bus
            .fields
            .into_iter()
            .map(crate::mappers::sensor_msgs::point_field::point_field_to_ros)
            .collect(),
        is_bigendian: bus.is_bigendian,
        point_step: bus.point_step as _,
        row_step: bus.row_step as _,
        data: crate::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
        is_dense: bus.is_dense,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsPointCloud2Mapper;

impl TypedTopicMapper for SensorMsgsPointCloud2Mapper {
    type Ros = ros_env::sensor_msgs::msg::PointCloud2;
    type Bus = robot_bus::sensor_msgs::msg::v1::PointCloud2;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(point_cloud2_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(point_cloud2_to_ros(msg))
    }
}
