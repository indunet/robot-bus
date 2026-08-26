//! Builtin [`TypedServiceMapper`] codecs (Trigger / SetBool).

use prost::Message as ProstMessage;

use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::TypedServiceMapper;
use crate::ros2_bridge::mappers::service;
use crate::ros2_bridge::vendor::std_srvs::srv as ros_srv;
use crate::std_srvs::srv::v1::{
    SetBoolRequest as BusSetBoolRequest, SetBoolResponse as BusSetBoolResponse,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};

/// Builtin codec for `std_srvs/srv/Trigger`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TriggerServiceMapper;

/// Builtin codec for `std_srvs/srv/SetBool`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SetBoolServiceMapper;

impl TypedServiceMapper for TriggerServiceMapper {
    type Ros = ros_srv::Trigger;

    fn type_name(&self) -> &str {
        "std_srvs/srv/Trigger"
    }

    fn ros_req_to_bus(&self, req: &ros_srv::Trigger_Request) -> Result<Vec<u8>> {
        Ok(service::trigger_ros_req_to_bus(req).encode_to_vec())
    }

    fn bus_req_to_ros(&self, payload: &[u8]) -> Result<ros_srv::Trigger_Request> {
        let bus = BusTriggerRequest::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode TriggerRequest: {e}")))?;
        Ok(service::trigger_bus_req_to_ros(&bus))
    }

    fn ros_resp_to_bus(&self, resp: &ros_srv::Trigger_Response) -> Result<Vec<u8>> {
        Ok(service::trigger_ros_resp_to_bus(resp).encode_to_vec())
    }

    fn bus_resp_to_ros(&self, payload: &[u8]) -> Result<ros_srv::Trigger_Response> {
        let bus = BusTriggerResponse::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode TriggerResponse: {e}")))?;
        Ok(service::trigger_bus_resp_to_ros(&bus))
    }

    fn error_response(&self, message: &str) -> ros_srv::Trigger_Response {
        ros_srv::Trigger_Response {
            success: false,
            message: message.into(),
        }
    }
}

impl TypedServiceMapper for SetBoolServiceMapper {
    type Ros = ros_srv::SetBool;

    fn type_name(&self) -> &str {
        "std_srvs/srv/SetBool"
    }

    fn ros_req_to_bus(&self, req: &ros_srv::SetBool_Request) -> Result<Vec<u8>> {
        Ok(service::set_bool_ros_req_to_bus(req).encode_to_vec())
    }

    fn bus_req_to_ros(&self, payload: &[u8]) -> Result<ros_srv::SetBool_Request> {
        let bus = BusSetBoolRequest::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode SetBoolRequest: {e}")))?;
        Ok(service::set_bool_bus_req_to_ros(&bus))
    }

    fn ros_resp_to_bus(&self, resp: &ros_srv::SetBool_Response) -> Result<Vec<u8>> {
        Ok(service::set_bool_ros_resp_to_bus(resp).encode_to_vec())
    }

    fn bus_resp_to_ros(&self, payload: &[u8]) -> Result<ros_srv::SetBool_Response> {
        let bus = BusSetBoolResponse::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode SetBoolResponse: {e}")))?;
        Ok(service::set_bool_bus_resp_to_ros(&bus))
    }

    fn error_response(&self, message: &str) -> ros_srv::SetBool_Response {
        ros_srv::SetBool_Response {
            success: false,
            message: message.into(),
        }
    }
}
