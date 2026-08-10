//! Mapper for `sensor_msgs/msg/RegionOfInterest`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn region_of_interest_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::RegionOfInterest> {
    Ok(crate::sensor_msgs::msg::v1::RegionOfInterest {
        x_offset: read_u32(view, "x_offset")?,
        y_offset: read_u32(view, "y_offset")?,
        height: read_u32(view, "height")?,
        width: read_u32(view, "width")?,
        do_rectify: read_bool(view, "do_rectify")?,
    })
}

pub(crate) fn region_of_interest_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::RegionOfInterest,
) -> Result<()> {
    write_u32(view, "x_offset", bus.x_offset)?;
    write_u32(view, "y_offset", bus.y_offset)?;
    write_u32(view, "height", bus.height)?;
    write_u32(view, "width", bus.width)?;
    write_bool(view, "do_rectify", bus.do_rectify)?;
    Ok(())
}

pub(crate) fn region_of_interest_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::RegionOfInterest> {
    region_of_interest_from_view(&msg.view())
}

pub(crate) fn region_of_interest_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::RegionOfInterest,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/RegionOfInterest")?;
    region_of_interest_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsRegionOfInterestMapper;
impl TopicMapper for SensorMsgsRegionOfInterestMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/RegionOfInterest"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(region_of_interest_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::RegionOfInterest as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode sensor_msgs/msg/RegionOfInterest: {e}"))
            })?;
        region_of_interest_bus_to_dyn(&bus)
    }
}
