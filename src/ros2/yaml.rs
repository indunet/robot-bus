//! YAML config → [`Ros2BridgeBuilder`].

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::errors::{BusError, Result};

use super::builder::{Direction, MsgKind, Ros2Bridge, Ros2BridgeBuilder};

#[derive(Debug, Deserialize)]
struct FileConfig {
    robot_bus: Option<RobotBusSection>,
    routes: Vec<RouteSection>,
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

fn default_direction() -> String {
    "both".into()
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

    if cfg.routes.is_empty() {
        return Err(BusError::Protocol("yaml routes must be non-empty".into()));
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
        let kind = MsgKind::parse(&route.type_name)
            .map_err(|e| BusError::Protocol(format!("routes[{i}]: {e}")))?;
        let direction = parse_direction(&route.direction)?;
        builder = builder.push_route(route.ros_topic, route.bus_topic, kind, direction);
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
