//! Mapper for `geometry_msgs/msg/PolygonInstance`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn polygon_instance_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::PolygonInstance> {
    Ok(crate::geometry_msgs::msg::v1::PolygonInstance {
        polygon: nested_view(view, "polygon")?
            .as_ref()
            .map(super::polygon::polygon_from_view)
            .transpose()?,
        id: read_i64(view, "id")?,
    })
}

pub(crate) fn polygon_instance_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::PolygonInstance,
) -> Result<()> {
    if let Some(v) = &bus.polygon {
        with_nested_mut(view, "polygon", |nested| super::polygon::polygon_write(nested, v))?;
    }
    write_i64(view, "id", bus.id)?;
    Ok(())
}

pub(crate) fn polygon_instance_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::PolygonInstance> {
    polygon_instance_from_view(&msg.view())
}

pub(crate) fn polygon_instance_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::PolygonInstance,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/PolygonInstance")?;
    polygon_instance_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPolygonInstanceMapper;
impl TopicMapper for GeometryMsgsPolygonInstanceMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PolygonInstance"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(polygon_instance_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::PolygonInstance as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode geometry_msgs/msg/PolygonInstance: {e}"))
            })?;
        polygon_instance_bus_to_dyn(&bus)
    }
}
