//! Mapper for `sensor_msgs/msg/PointCloud2`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn point_cloud2_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::PointCloud2> {
    Ok(crate::sensor_msgs::msg::v1::PointCloud2 {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        height: read_u32(view, "height")?,
        width: read_u32(view, "width")?,
        fields: read_message_seq(view, "fields", super::point_field::point_field_from_view)?,
        is_bigendian: read_bool(view, "is_bigendian")?,
        point_step: read_u32(view, "point_step")?,
        row_step: read_u32(view, "row_step")?,
        data: read_byte_seq(view, "data")?,
        is_dense: read_bool(view, "is_dense")?,
    })
}

pub(crate) fn point_cloud2_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::PointCloud2,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "height", bus.height)?;
    write_u32(view, "width", bus.width)?;
    write_message_seq(view, "fields", &bus.fields, super::point_field::point_field_write)?;
    write_bool(view, "is_bigendian", bus.is_bigendian)?;
    write_u32(view, "point_step", bus.point_step)?;
    write_u32(view, "row_step", bus.row_step)?;
    write_byte_seq(view, "data", &bus.data)?;
    write_bool(view, "is_dense", bus.is_dense)?;
    Ok(())
}

pub(crate) fn point_cloud2_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::PointCloud2> {
    point_cloud2_from_view(&msg.view())
}

pub(crate) fn point_cloud2_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::PointCloud2,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/PointCloud2")?;
    point_cloud2_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsPointCloud2Mapper;
impl TopicMapper for SensorMsgsPointCloud2Mapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/PointCloud2"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_cloud2_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::PointCloud2 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/PointCloud2: {e}")))?;
        point_cloud2_bus_to_dyn(&bus)
    }

    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: crate::runtime::TopicPublisherRaw,
        ros_topic: &str,
        qos: crate::ros2_bridge::mapper::TopicRouteQos,
    ) -> Result<Option<Box<dyn std::any::Any + Send + Sync>>> {
        typed_point_cloud2_subscription(ros_node, bus_pub, ros_topic, qos)
    }
}

#[cfg(feature = "ros2-shim")]
fn typed_point_cloud2_subscription(
    _ros_node: &rclrs::Node,
    _bus_pub: crate::runtime::TopicPublisherRaw,
    _ros_topic: &str,
    _qos: crate::ros2_bridge::mapper::TopicRouteQos,
) -> Result<Option<Box<dyn std::any::Any + Send + Sync>>> {
    Ok(None)
}

#[cfg(not(feature = "ros2-shim"))]
fn typed_point_cloud2_subscription(
    ros_node: &rclrs::Node,
    bus_pub: crate::runtime::TopicPublisherRaw,
    ros_topic: &str,
    qos: crate::ros2_bridge::mapper::TopicRouteQos,
) -> Result<Option<Box<dyn std::any::Any + Send + Sync>>> {
    use crate::ros2_bridge::mapper::ros_topic_options;
    use prost::Message as _;
    use ros_env::sensor_msgs::msg::PointCloud2 as RosPc2;

    let opts = ros_topic_options(ros_topic, qos);
    let sub = ros_node
        .create_subscription(opts, move |msg: RosPc2| {
            let payload = point_cloud2_typed_to_bus(&msg).encode_to_vec();
            if let Err(e) = bus_pub.publish(&payload) {
                log::warn!("ros→bus sensor_msgs/msg/PointCloud2 publish: {e}");
            }
        })
        .map_err(|e| BusError::Protocol(format!("ros PointCloud2 subscription: {e}")))?;
    Ok(Some(Box::new(sub)))
}

#[cfg(not(feature = "ros2-shim"))]
fn point_cloud2_typed_to_bus(
    msg: &ros_env::sensor_msgs::msg::PointCloud2,
) -> crate::sensor_msgs::msg::v1::PointCloud2 {
    crate::sensor_msgs::msg::v1::PointCloud2 {
        header: Some(crate::std_msgs::msg::v1::Header {
            stamp: Some(crate::builtin_interfaces::msg::v1::Time {
                sec: msg.header.stamp.sec,
                nanosec: msg.header.stamp.nanosec,
            }),
            frame_id: msg.header.frame_id.to_string(),
        }),
        height: msg.height,
        width: msg.width,
        fields: msg
            .fields
            .iter()
            .map(|f| crate::sensor_msgs::msg::v1::PointField {
                name: f.name.to_string(),
                offset: f.offset,
                datatype: u32::from(f.datatype),
                count: f.count,
            })
            .collect(),
        is_bigendian: msg.is_bigendian,
        point_step: msg.point_step,
        row_step: msg.row_step,
        data: msg.data.iter().copied().collect(),
        is_dense: msg.is_dense,
    }
}
