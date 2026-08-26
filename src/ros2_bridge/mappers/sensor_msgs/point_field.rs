//! Mapper for `sensor_msgs/msg/PointField`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point_field_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::PointField> {
    Ok(crate::sensor_msgs::msg::v1::PointField {
        name: read_string(view, "name")?,
        offset: read_u32(view, "offset")?,
        datatype: read_u32(view, "datatype")?,
        count: read_u32(view, "count")?,
    })
}

pub(crate) fn point_field_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::PointField,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    write_u32(view, "offset", bus.offset)?;
    write_u32(view, "datatype", bus.datatype)?;
    write_u32(view, "count", bus.count)?;
    Ok(())
}

pub(crate) fn point_field_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::PointField> {
    point_field_from_view(&msg.view())
}

pub(crate) fn point_field_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::PointField,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/PointField")?;
    point_field_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsPointFieldMapper;
impl TopicMapper for SensorMsgsPointFieldMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/PointField"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_field_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::PointField as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/PointField: {e}")))?;
        point_field_bus_to_dyn(&bus)
    }
}
