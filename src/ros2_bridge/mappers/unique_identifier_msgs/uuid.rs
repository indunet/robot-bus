//! Typed mapper for `unique_identifier_msgs/msg/UUID`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn uuid_to_bus(msg: ros_env::unique_identifier_msgs::msg::UUID) -> crate::unique_identifier_msgs::msg::v1::Uuid {
    crate::unique_identifier_msgs::msg::v1::Uuid {
        uuid: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.uuid),
    }
}

pub(crate) fn uuid_to_ros(bus: crate::unique_identifier_msgs::msg::v1::Uuid) -> ros_env::unique_identifier_msgs::msg::UUID {
    ros_env::unique_identifier_msgs::msg::UUID {
        uuid: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.uuid),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UniqueIdentifierMsgsUuidMapper;

impl TypedTopicMapper for UniqueIdentifierMsgsUuidMapper {
    type Ros = ros_env::unique_identifier_msgs::msg::UUID;
    type Bus = crate::unique_identifier_msgs::msg::v1::Uuid;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(uuid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(uuid_to_ros(msg))
    }
}
