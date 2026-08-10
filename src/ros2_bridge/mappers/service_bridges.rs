//! Builtin [`ServiceMapper`] implementations (Trigger / SetBool).

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use prost::Message as ProstMessage;

use crate::errors::{BusError, Result};
use crate::runtime::ServiceHandler;
use crate::ros2_bridge::mapper::{Direction, ServiceMapper, ServiceWireContext};
use crate::ros2_bridge::vendor::std_srvs::srv as ros_srv;
use crate::std_srvs::srv::v1::{
    SetBool as BusSetBool, SetBoolRequest as BusSetBoolRequest,
    SetBoolResponse as BusSetBoolResponse, Trigger as BusTrigger,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};

use super::service;

pub struct TriggerServiceMapper;
pub struct SetBoolServiceMapper;

pub fn lookup_service_mapper(type_name: &str) -> Result<Arc<dyn ServiceMapper>> {
    match type_name {
        "std_srvs/srv/Trigger" => Ok(Arc::new(TriggerServiceMapper)),
        "std_srvs/srv/SetBool" => Ok(Arc::new(SetBoolServiceMapper)),
        other => Err(BusError::Protocol(format!(
            "unsupported ros2 bridge service type {other:?}; \
             builtins: std_srvs/srv/Trigger, std_srvs/srv/SetBool; \
             for custom types use .mapper(...) on the service route"
        ))),
    }
}

fn wait_service_ready(
    client_ready: impl Fn() -> bool,
    timeout: Duration,
) -> std::result::Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if client_ready() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("timed out waiting for ROS service".into())
}

fn call_ros_trigger(
    client: &rclrs::Client<ros_srv::Trigger>,
    bus_req: &BusTriggerRequest,
    timeout: Duration,
) -> std::result::Result<BusTriggerResponse, String> {
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let ros_req = service::trigger_bus_req_to_ros(bus_req);
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: ros_srv::Trigger_Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros Trigger call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(service::trigger_ros_resp_to_bus(&resp)),
        Err(_) => Err("timed out waiting for ROS Trigger response".into()),
    }
}

fn call_ros_set_bool(
    client: &rclrs::Client<ros_srv::SetBool>,
    bus_req: &BusSetBoolRequest,
    timeout: Duration,
) -> std::result::Result<BusSetBoolResponse, String> {
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let ros_req = service::set_bool_bus_req_to_ros(bus_req);
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: ros_srv::SetBool_Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros SetBool call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(service::set_bool_ros_resp_to_bus(&resp)),
        Err(_) => Err("timed out waiting for ROS SetBool response".into()),
    }
}

impl ServiceMapper for TriggerServiceMapper {
    fn type_name(&self) -> &'static str {
        "std_srvs/srv/Trigger"
    }

    fn wire(&self, ctx: ServiceWireContext<'_>) -> Result<()> {
        match ctx.direction {
            Direction::Ros2ToBus => {
                let bus_client = Arc::new(Mutex::new(
                    ctx.bus_node
                        .create_client::<BusTrigger>(ctx.bus_service)?,
                ));
                let timeout = ctx.timeout;
                let srv = ctx
                    .ros_node
                    .create_service::<ros_srv::Trigger, _>(
                        ctx.ros_service,
                        move |_req: ros_srv::Trigger_Request| {
                            let bus_req = service::trigger_ros_req_to_bus(&_req);
                            let guard = match bus_client.lock() {
                                Ok(g) => g,
                                Err(e) => {
                                    return ros_srv::Trigger_Response {
                                        success: false,
                                        message: format!("bus client lock poisoned: {e}"),
                                    };
                                }
                            };
                            match guard.call(&bus_req, Some(timeout)) {
                                Ok(bus_resp) => service::trigger_bus_resp_to_ros(&bus_resp),
                                Err(e) => ros_srv::Trigger_Response {
                                    success: false,
                                    message: format!("bus call failed: {e}"),
                                },
                            }
                        },
                    )
                    .map_err(|e| BusError::Protocol(format!("ros create_service Trigger: {e}")))?;
                ctx.ros_entities.push(Box::new(srv));
            }
            Direction::BusToRos2 => {
                let ros_client = ctx
                    .ros_node
                    .create_client::<ros_srv::Trigger>(ctx.ros_service)
                    .map_err(|e| BusError::Protocol(format!("ros create_client Trigger: {e}")))?;
                ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
                let timeout = ctx.timeout;
                let handler: ServiceHandler = Arc::new(move |body| {
                    let bus_req = match BusTriggerRequest::decode(body) {
                        Ok(r) => r,
                        Err(e) => {
                            return BusTriggerResponse {
                                success: false,
                                message: format!("decode TriggerRequest: {e}"),
                            }
                            .encode_to_vec();
                        }
                    };
                    match call_ros_trigger(&ros_client, &bus_req, timeout) {
                        Ok(resp) => resp.encode_to_vec(),
                        Err(msg) => BusTriggerResponse {
                            success: false,
                            message: msg,
                        }
                        .encode_to_vec(),
                    }
                });
                let _ = ctx
                    .bus_node
                    .create_service_raw(ctx.bus_service, handler, None)?;
            }
        }
        Ok(())
    }
}

impl ServiceMapper for SetBoolServiceMapper {
    fn type_name(&self) -> &'static str {
        "std_srvs/srv/SetBool"
    }

    fn wire(&self, ctx: ServiceWireContext<'_>) -> Result<()> {
        match ctx.direction {
            Direction::Ros2ToBus => {
                let bus_client = Arc::new(Mutex::new(
                    ctx.bus_node
                        .create_client::<BusSetBool>(ctx.bus_service)?,
                ));
                let timeout = ctx.timeout;
                let srv = ctx
                    .ros_node
                    .create_service::<ros_srv::SetBool, _>(
                        ctx.ros_service,
                        move |req: ros_srv::SetBool_Request| {
                            let bus_req = service::set_bool_ros_req_to_bus(&req);
                            let guard = match bus_client.lock() {
                                Ok(g) => g,
                                Err(e) => {
                                    return ros_srv::SetBool_Response {
                                        success: false,
                                        message: format!("bus client lock poisoned: {e}"),
                                    };
                                }
                            };
                            match guard.call(&bus_req, Some(timeout)) {
                                Ok(bus_resp) => service::set_bool_bus_resp_to_ros(&bus_resp),
                                Err(e) => ros_srv::SetBool_Response {
                                    success: false,
                                    message: format!("bus call failed: {e}"),
                                },
                            }
                        },
                    )
                    .map_err(|e| BusError::Protocol(format!("ros create_service SetBool: {e}")))?;
                ctx.ros_entities.push(Box::new(srv));
            }
            Direction::BusToRos2 => {
                let ros_client = ctx
                    .ros_node
                    .create_client::<ros_srv::SetBool>(ctx.ros_service)
                    .map_err(|e| BusError::Protocol(format!("ros create_client SetBool: {e}")))?;
                ctx.ros_entities.push(Box::new(Arc::clone(&ros_client)));
                let timeout = ctx.timeout;
                let handler: ServiceHandler = Arc::new(move |body| {
                    let bus_req = match BusSetBoolRequest::decode(body) {
                        Ok(r) => r,
                        Err(e) => {
                            return BusSetBoolResponse {
                                success: false,
                                message: format!("decode SetBoolRequest: {e}"),
                            }
                            .encode_to_vec();
                        }
                    };
                    match call_ros_set_bool(&ros_client, &bus_req, timeout) {
                        Ok(resp) => resp.encode_to_vec(),
                        Err(msg) => BusSetBoolResponse {
                            success: false,
                            message: msg,
                        }
                        .encode_to_vec(),
                    }
                });
                let _ = ctx
                    .bus_node
                    .create_service_raw(ctx.bus_service, handler, None)?;
            }
        }
        Ok(())
    }
}
