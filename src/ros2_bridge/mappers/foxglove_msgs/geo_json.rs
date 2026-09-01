//! Typed mapper for `foxglove_msgs/msg/GeoJSON`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn geo_json_to_bus(msg: ros_env::foxglove_msgs::msg::GeoJSON) -> crate::foxglove_msgs::msg::v1::GeoJson {
    crate::foxglove_msgs::msg::v1::GeoJson {
        geojson: crate::ros2_bridge::mappers::convert::from_ros_string(msg.geojson),
    }
}

pub(crate) fn geo_json_to_ros(bus: crate::foxglove_msgs::msg::v1::GeoJson) -> ros_env::foxglove_msgs::msg::GeoJSON {
    ros_env::foxglove_msgs::msg::GeoJSON {
        geojson: crate::ros2_bridge::mappers::convert::to_ros_string(bus.geojson),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsGeoJsonMapper;

impl TypedTopicMapper for FoxgloveMsgsGeoJsonMapper {
    type Ros = ros_env::foxglove_msgs::msg::GeoJSON;
    type Bus = crate::foxglove_msgs::msg::v1::GeoJson;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(geo_json_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(geo_json_to_ros(msg))
    }
}
