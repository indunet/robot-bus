//! YAML config → [`Ros2BridgeBuilder`].

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::errors::{BusError, Result};

use super::builder::{ActKind, Direction, Ros2Bridge, Ros2BridgeBuilder, SrvKind};
use super::codec;

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
    #[serde(default)]
    domain_id: u32,
    #[serde(default = "default_timeout")]
    timeout: f64,
    broker_id: Option<String>,
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
    "both".into()
}

fn default_service_direction() -> String {
    "ros_to_bus".into()
}

pub fn builder_from_yaml(path: impl AsRef<Path>) -> Result<Ros2BridgeBuilder> {
    let text = fs::read_to_string(path.as_ref()).map_err(|e| {
        BusError::Protocol(format!(
            "read ros2 bridge yaml {}: {e}",
            path.as_ref().display()
        ))
    })?;
    let cfg: FileConfig = serde_yaml::from_str(&text)
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
            let domain_id = rb.discover.as_ref().map(|d| d.domain_id).unwrap_or(0);
            let timeout = rb.discover.as_ref().map(|d| d.timeout);
            let broker_id = rb.discover.and_then(|d| d.broker_id);
            builder.bus_discover_ex(domain_id, timeout, broker_id)?
        }
        other => {
            return Err(BusError::Protocol(format!(
                "robot_bus.transport must be tcp | ipc | discover, got {other:?}"
            )))
        }
    };

    for (i, route) in cfg.routes.into_iter().enumerate() {
        codec::lookup_topic_codec(&route.type_name)
            .map_err(|e| BusError::Protocol(format!("routes[{i}]: {e}")))?;
        let direction = parse_direction(&route.direction)?;
        builder = builder
            .push_route(route.ros_topic, route.bus_topic, route.type_name, direction)
            .map_err(|e| BusError::Protocol(format!("routes[{i}]: {e}")))?;
    }

    for (i, svc) in cfg.services.into_iter().enumerate() {
        let kind = SrvKind::parse(&svc.type_name)
            .map_err(|e| BusError::Protocol(format!("services[{i}]: {e}")))?;
        let direction = parse_service_direction(&svc.direction)?;
        builder = builder
            .push_service(svc.ros_service, svc.bus_service, kind, direction)
            .map_err(|e| BusError::Protocol(format!("services[{i}]: {e}")))?;
    }

    for (i, act) in cfg.actions.into_iter().enumerate() {
        let kind = ActKind::parse(&act.type_name)
            .map_err(|e| BusError::Protocol(format!("actions[{i}]: {e}")))?;
        let direction = parse_action_direction(&act.direction)?;
        builder = builder
            .push_action(act.ros_action, act.bus_action, kind, direction)
            .map_err(|e| BusError::Protocol(format!("actions[{i}]: {e}")))?;
    }

    Ok(builder)
}

fn parse_direction(s: &str) -> Result<Direction> {
    match s {
        "ros_to_bus" => Ok(Direction::RosToBus),
        "bus_to_ros" => Ok(Direction::BusToRos),
        "both" => Ok(Direction::Both),
        other => Err(BusError::Protocol(format!(
            "direction must be ros_to_bus | bus_to_ros | both, got {other:?}"
        ))),
    }
}

fn parse_service_direction(s: &str) -> Result<Direction> {
    match s {
        "ros_to_bus" => Ok(Direction::RosToBus),
        "bus_to_ros" => Ok(Direction::BusToRos),
        "both" => Err(BusError::Protocol(
            "service direction must be ros_to_bus | bus_to_ros (both is not supported)".into(),
        )),
        other => Err(BusError::Protocol(format!(
            "service direction must be ros_to_bus | bus_to_ros, got {other:?}"
        ))),
    }
}

fn parse_action_direction(s: &str) -> Result<Direction> {
    match s {
        "ros_to_bus" => Ok(Direction::RosToBus),
        "bus_to_ros" => Ok(Direction::BusToRos),
        "both" => Err(BusError::Protocol(
            "action direction must be ros_to_bus | bus_to_ros (both is not supported)".into(),
        )),
        other => Err(BusError::Protocol(format!(
            "action direction must be ros_to_bus | bus_to_ros, got {other:?}"
        ))),
    }
}
