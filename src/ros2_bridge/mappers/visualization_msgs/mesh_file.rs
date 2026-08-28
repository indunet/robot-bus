//! Typed mapper for `visualization_msgs/msg/MeshFile`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn mesh_file_to_bus(msg: ros_env::visualization_msgs::msg::MeshFile) -> crate::visualization_msgs::msg::v1::MeshFile {
    crate::visualization_msgs::msg::v1::MeshFile {
        filename: crate::ros2_bridge::mappers::convert::from_ros_string(msg.filename),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn mesh_file_to_ros(bus: crate::visualization_msgs::msg::v1::MeshFile) -> ros_env::visualization_msgs::msg::MeshFile {
    ros_env::visualization_msgs::msg::MeshFile {
        filename: crate::ros2_bridge::mappers::convert::to_ros_string(bus.filename),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsMeshFileMapper;

impl TypedTopicMapper for VisualizationMsgsMeshFileMapper {
    type Ros = ros_env::visualization_msgs::msg::MeshFile;
    type Bus = crate::visualization_msgs::msg::v1::MeshFile;

    fn type_name(&self) -> &'static str {
        "visualization_msgs/msg/MeshFile"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(mesh_file_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(mesh_file_to_ros(msg))
    }
}
