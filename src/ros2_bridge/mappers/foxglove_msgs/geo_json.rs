//! Mapper for `foxglove_msgs/msg/GeoJSON`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn geo_json_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::GeoJson> {
    Ok(crate::foxglove_msgs::msg::v1::GeoJson {
        geojson: read_string(view, "geojson")?,
    })
}

pub(crate) fn geo_json_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::GeoJson,
) -> Result<()> {
    write_string(view, "geojson", &bus.geojson)?;
    Ok(())
}

pub(crate) fn geo_json_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::GeoJson> {
    geo_json_from_view(&msg.view())
}

pub(crate) fn geo_json_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::GeoJson,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/GeoJSON")?;
    geo_json_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsGeoJsonMapper;
impl TopicMapper for FoxgloveMsgsGeoJsonMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/GeoJSON"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(geo_json_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::GeoJson as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/GeoJSON: {e}")))?;
        geo_json_bus_to_dyn(&bus)
    }
}
