//! Mapper for `sensor_msgs/msg/Image`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn image_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Image> {
    Ok(crate::sensor_msgs::msg::v1::Image {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        height: read_u32(view, "height")?,
        width: read_u32(view, "width")?,
        encoding: read_string(view, "encoding")?,
        is_bigendian: read_bool(view, "is_bigendian")?,
        step: read_u32(view, "step")?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn image_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Image,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "height", bus.height)?;
    write_u32(view, "width", bus.width)?;
    write_string(view, "encoding", &bus.encoding)?;
    write_bool(view, "is_bigendian", bus.is_bigendian)?;
    write_u32(view, "step", bus.step)?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn image_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Image> {
    image_from_view(&msg.view())
}

pub(crate) fn image_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Image,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Image")?;
    image_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsImageMapper;
impl TopicMapper for SensorMsgsImageMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Image"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Image as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Image: {e}")))?;
        image_bus_to_dyn(&bus)
    }

    fn create_ros2_to_bus_subscription(
        &self,
        ros_node: &rclrs::Node,
        bus_pub: crate::runtime::TopicPublisherRaw,
        ros_topic: &str,
        qos: crate::ros2_bridge::mapper::TopicRouteQos,
    ) -> Result<Option<Box<dyn std::any::Any + Send + Sync>>> {
        typed_image_subscription(ros_node, bus_pub, ros_topic, qos)
    }
}

#[cfg(feature = "ros2-shim")]
fn typed_image_subscription(
    _ros_node: &rclrs::Node,
    _bus_pub: crate::runtime::TopicPublisherRaw,
    _ros_topic: &str,
    _qos: crate::ros2_bridge::mapper::TopicRouteQos,
) -> Result<Option<Box<dyn std::any::Any + Send + Sync>>> {
    Ok(None)
}

#[cfg(not(feature = "ros2-shim"))]
fn typed_image_subscription(
    ros_node: &rclrs::Node,
    bus_pub: crate::runtime::TopicPublisherRaw,
    ros_topic: &str,
    qos: crate::ros2_bridge::mapper::TopicRouteQos,
) -> Result<Option<Box<dyn std::any::Any + Send + Sync>>> {
    use crate::ros2_bridge::mapper::ros_topic_options;
    use prost::Message as _;
    use ros_env::sensor_msgs::msg::Image as RosImage;

    let opts = ros_topic_options(ros_topic, qos);
    let sub = ros_node
        .create_subscription(opts, move |msg: RosImage| {
            let payload = image_typed_to_bus(&msg).encode_to_vec();
            if let Err(e) = bus_pub.publish(&payload) {
                log::warn!("ros→bus sensor_msgs/msg/Image publish: {e}");
            }
        })
        .map_err(|e| BusError::Protocol(format!("ros Image subscription: {e}")))?;
    Ok(Some(Box::new(sub)))
}

#[cfg(not(feature = "ros2-shim"))]
fn image_typed_to_bus(msg: &ros_env::sensor_msgs::msg::Image) -> crate::sensor_msgs::msg::v1::Image {
    crate::sensor_msgs::msg::v1::Image {
        header: Some(crate::std_msgs::msg::v1::Header {
            stamp: Some(crate::builtin_interfaces::msg::v1::Time {
                sec: msg.header.stamp.sec,
                nanosec: msg.header.stamp.nanosec,
            }),
            frame_id: msg.header.frame_id.to_string(),
        }),
        height: msg.height,
        width: msg.width,
        encoding: msg.encoding.to_string(),
        is_bigendian: msg.is_bigendian != 0,
        step: msg.step,
        data: msg.data.iter().copied().collect(),
    }
}
