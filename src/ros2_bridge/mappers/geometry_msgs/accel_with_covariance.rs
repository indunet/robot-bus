//! Typed mapper for `geometry_msgs/msg/AccelWithCovariance`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn accel_with_covariance_to_bus(msg: ros_env::geometry_msgs::msg::AccelWithCovariance) -> crate::geometry_msgs::msg::v1::AccelWithCovariance {
    crate::geometry_msgs::msg::v1::AccelWithCovariance {
        accel: Some(crate::ros2_bridge::mappers::geometry_msgs::accel::accel_to_bus(msg.accel)),
        covariance: crate::ros2_bridge::mappers::convert::f64_seq(msg.covariance),
    }
}

pub(crate) fn accel_with_covariance_to_ros(bus: crate::geometry_msgs::msg::v1::AccelWithCovariance) -> ros_env::geometry_msgs::msg::AccelWithCovariance {
    ros_env::geometry_msgs::msg::AccelWithCovariance {
        accel: crate::ros2_bridge::mappers::geometry_msgs::accel::accel_to_ros(bus.accel.unwrap_or_default()),
        covariance: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.covariance),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsAccelWithCovarianceMapper;

impl TypedTopicMapper for GeometryMsgsAccelWithCovarianceMapper {
    type Ros = ros_env::geometry_msgs::msg::AccelWithCovariance;
    type Bus = crate::geometry_msgs::msg::v1::AccelWithCovariance;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/AccelWithCovariance"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(accel_with_covariance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(accel_with_covariance_to_ros(msg))
    }
}
