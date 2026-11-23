//! Typed mapper for `unique_identifier_msgs/msg/UUID`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn uuid_to_bus(
    msg: ros_env::unique_identifier_msgs::msg::UUID,
) -> robot_bus::unique_identifier_msgs::msg::v1::Uuid {
    robot_bus::unique_identifier_msgs::msg::v1::Uuid {
        uuid: crate::mappers::convert::IntoU8Vec::into_u8_vec(msg.uuid),
    }
}

pub(crate) fn uuid_to_ros(
    bus: robot_bus::unique_identifier_msgs::msg::v1::Uuid,
) -> ros_env::unique_identifier_msgs::msg::UUID {
    ros_env::unique_identifier_msgs::msg::UUID {
        uuid: crate::mappers::convert::FromByteSeq::from_byte_seq(bus.uuid),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UniqueIdentifierMsgsUuidMapper;

impl TypedTopicMapper for UniqueIdentifierMsgsUuidMapper {
    type Ros = ros_env::unique_identifier_msgs::msg::UUID;
    type Bus = robot_bus::unique_identifier_msgs::msg::v1::Uuid;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(uuid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(uuid_to_ros(msg))
    }
}
