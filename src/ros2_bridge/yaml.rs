//! YAML config → [`Ros2BridgeBuilder`].

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::errors::{BusError, Result};

use super::builder::{Ros2Bridge, Ros2BridgeBuilder};
use super::mapper::Direction;

#[derive(Debug, Deserialize)]
struct FileConfig {
    robot_bus: Option<RobotBusSection>,
    #[serde(default)]
    routes: Vec<RouteSection>,
    #[serde(default)]
    services: Vec<ServiceSection>,
    #[serde(default)]
    actions: Vec<ActionSection>,
}

#[derive(Debug, Deserialize)]
struct RobotBusSection {
    #[serde(default = "default_transport")]
    transport: String,
    #[serde(default = "default_host")]
    host: String,
    ipc_path: Option<String>,
    discover: Option<DiscoverSection>,
}

fn default_transport() -> String {
    "tcp".into()
}
fn default_host() -> String {
    "localhost".into()
}

#[derive(Debug, Deserialize)]
struct DiscoverSection {
    #[serde(default = "default_api_url")]
    api_url: String,
    #[serde(default = "default_timeout")]
    timeout: f64,
    broker_id: Option<String>,
}

fn default_api_url() -> String {
    format!("http://127.0.0.1:{}", crate::transports::DEFAULT_API_PORT)
}

fn default_timeout() -> f64 {
    3.0
}

#[derive(Debug, Deserialize)]
struct RouteSection {
    ros_topic: String,
    bus_topic: String,
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default = "default_direction")]
    direction: String,
}

#[derive(Debug, Deserialize)]
struct ServiceSection {
    ros_service: String,
    bus_service: String,
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default = "default_service_direction")]
    direction: String,
}

#[derive(Debug, Deserialize)]
struct ActionSection {
    ros_action: String,
    bus_action: String,
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default = "default_service_direction")]
    direction: String,
}

fn default_direction() -> String {
    "ros2_to_bus".into()
}

fn default_service_direction() -> String {
    "ros2_to_bus".into()
}

pub fn builder_from_yaml(path: impl AsRef<Path>) -> Result<Ros2BridgeBuilder> {
    let text = fs::read_to_string(path.as_ref()).map_err(|e| {
        BusError::Protocol(format!(
            "read ros2 bridge yaml {}: {e}",
            path.as_ref().display()
        ))
    })?;
    builder_from_yaml_str(&text)
}

fn builder_from_yaml_str(text: &str) -> Result<Ros2BridgeBuilder> {
    let cfg: FileConfig = serde_yaml::from_str(text)
        .map_err(|e| BusError::Protocol(format!("parse ros2 bridge yaml: {e}")))?;

    if cfg.routes.is_empty() && cfg.services.is_empty() && cfg.actions.is_empty() {
        return Err(BusError::Protocol(
            "yaml must include at least one routes[], services[], or actions[] entry".into(),
        ));
    }

    let rb = cfg.robot_bus.unwrap_or(RobotBusSection {
        transport: default_transport(),
        host: default_host(),
        ipc_path: None,
        discover: None,
    });

    let mut builder = Ros2Bridge::new("ros2_bridge");
    builder = match rb.transport.as_str() {
        "tcp" => builder.bus_tcp(rb.host),
        "ipc" => match rb.ipc_path {
            Some(p) => builder.bus_ipc_at(p),
            None => builder.bus_ipc(),
        },
        "discover" => {
            let api_url = rb
                .discover
                .as_ref()
                .map(|d| d.api_url.clone())
                .unwrap_or_else(default_api_url);
            let timeout = rb.discover.as_ref().map(|d| d.timeout);
            let broker_id = rb.discover.and_then(|d| d.broker_id);
            builder.bus_discover_ex(api_url, timeout, broker_id)?
        }
        other => {
            return Err(BusError::Protocol(format!(
                "robot_bus.transport must be tcp | ipc | discover, got {other:?}"
            )));
        }
    };

    for (i, route) in cfg.routes.into_iter().enumerate() {
        let direction = parse_direction(&route.direction)?;
        builder = builder
            .add_route(route.ros_topic, route.bus_topic, route.type_name, direction)
            .map_err(|e| BusError::Protocol(format!("routes[{i}]: {e}")))?;
    }

    for (i, svc) in cfg.services.into_iter().enumerate() {
        let direction = parse_service_direction(&svc.direction)?;
        builder = builder
            .add_service(svc.ros_service, svc.bus_service, svc.type_name, direction)
            .map_err(|e| BusError::Protocol(format!("services[{i}]: {e}")))?;
    }

    for (i, act) in cfg.actions.into_iter().enumerate() {
        let direction = parse_action_direction(&act.direction)?;
        builder = builder
            .add_action(act.ros_action, act.bus_action, act.type_name, direction)
            .map_err(|e| BusError::Protocol(format!("actions[{i}]: {e}")))?;
    }

    Ok(builder)
}

fn parse_direction(s: &str) -> Result<Direction> {
    match s {
        "ros2_to_bus" => Ok(Direction::Ros2ToBus),
        "bus_to_ros2" => Ok(Direction::BusToRos2),
        "both" => Err(BusError::Protocol(
            "topic direction must be ros2_to_bus | bus_to_ros2 (both is not supported)".into(),
        )),
        other => Err(BusError::Protocol(format!(
            "direction must be ros2_to_bus | bus_to_ros2, got {other:?}"
        ))),
    }
}

fn parse_service_direction(s: &str) -> Result<Direction> {
    match s {
        "ros2_to_bus" => Ok(Direction::Ros2ToBus),
        "bus_to_ros2" => Ok(Direction::BusToRos2),
        "both" => Err(BusError::Protocol(
            "service direction must be ros2_to_bus | bus_to_ros2 (both is not supported)".into(),
        )),
        other => Err(BusError::Protocol(format!(
            "service direction must be ros2_to_bus | bus_to_ros2, got {other:?}"
        ))),
    }
}

fn parse_action_direction(s: &str) -> Result<Direction> {
    match s {
        "ros2_to_bus" => Ok(Direction::Ros2ToBus),
        "bus_to_ros2" => Ok(Direction::BusToRos2),
        "both" => Err(BusError::Protocol(
            "action direction must be ros2_to_bus | bus_to_ros2 (both is not supported)".into(),
        )),
        other => Err(BusError::Protocol(format!(
            "action direction must be ros2_to_bus | bus_to_ros2, got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_camera_h264_bridge_yaml() {
        // Image → bus, H.264 CompressedVideo ← bus (needs rbus_image_encoder at runtime).
        let yaml = r#"
robot_bus:
  transport: tcp
  host: localhost

routes:
  - ros_topic: /camera/image_raw
    bus_topic: /camera/image_raw
    type: sensor_msgs/msg/Image
    direction: ros2_to_bus
  - ros_topic: /camera/video
    bus_topic: /camera/video
    type: foxglove_msgs/msg/CompressedVideo
    direction: bus_to_ros2
"#;
        builder_from_yaml_str(yaml).expect("camera h264 yaml should parse");
    }

    #[test]
    fn rejects_empty_yaml() {
        let err = builder_from_yaml_str("robot_bus:\n  transport: tcp\n")
            .err()
            .expect("empty routes should fail");
        assert!(err.to_string().contains("at least one"), "{err}");
    }
}
