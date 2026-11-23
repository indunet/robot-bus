//! Typed std_srvs request/response mapping used by Ros2Bridge services.

// --- std_srvs typed conversions (rclrs vendor ↔ bus prost) ---

use crate::ros2_bridge::vendor::std_srvs::srv as ros_srv;
use crate::std_srvs::srv::v1::{
    SetBoolRequest as BusSetBoolRequest, SetBoolResponse as BusSetBoolResponse,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};

pub fn trigger_ros_req_to_bus(_req: &ros_srv::Trigger_Request) -> BusTriggerRequest {
    BusTriggerRequest {}
}

pub fn trigger_bus_req_to_ros(_req: &BusTriggerRequest) -> ros_srv::Trigger_Request {
    ros_srv::Trigger_Request {
        structure_needs_at_least_one_member: 0,
    }
}

pub fn trigger_ros_resp_to_bus(resp: &ros_srv::Trigger_Response) -> BusTriggerResponse {
    BusTriggerResponse {
        success: resp.success,
        message: resp.message.clone(),
    }
}

pub fn trigger_bus_resp_to_ros(resp: &BusTriggerResponse) -> ros_srv::Trigger_Response {
    ros_srv::Trigger_Response {
        success: resp.success,
        message: resp.message.clone(),
    }
}

pub fn set_bool_ros_req_to_bus(req: &ros_srv::SetBool_Request) -> BusSetBoolRequest {
    BusSetBoolRequest { data: req.data }
}

pub fn set_bool_bus_req_to_ros(req: &BusSetBoolRequest) -> ros_srv::SetBool_Request {
    ros_srv::SetBool_Request { data: req.data }
}

pub fn set_bool_ros_resp_to_bus(resp: &ros_srv::SetBool_Response) -> BusSetBoolResponse {
    BusSetBoolResponse {
        success: resp.success,
        message: resp.message.clone(),
    }
}

pub fn set_bool_bus_resp_to_ros(resp: &BusSetBoolResponse) -> ros_srv::SetBool_Response {
    ros_srv::SetBool_Response {
        success: resp.success,
        message: resp.message.clone(),
    }
}

#[cfg(test)]
mod service_convert_tests {
    use super::*;

    #[test]
    fn trigger_roundtrip_fields() {
        let ros = ros_srv::Trigger_Response {
            success: true,
            message: "ok".into(),
        };
        let bus = trigger_ros_resp_to_bus(&ros);
        assert!(bus.success);
        assert_eq!(bus.message, "ok");
        let back = trigger_bus_resp_to_ros(&bus);
        assert!(back.success);
        assert_eq!(back.message, "ok");
    }

    #[test]
    fn set_bool_roundtrip_fields() {
        let ros_req = ros_srv::SetBool_Request { data: true };
        let bus = set_bool_ros_req_to_bus(&ros_req);
        assert!(bus.data);
        let back = set_bool_bus_req_to_ros(&bus);
        assert!(back.data);

        let ros_resp = ros_srv::SetBool_Response {
            success: false,
            message: "no".into(),
        };
        let bus_resp = set_bool_ros_resp_to_bus(&ros_resp);
        assert!(!bus_resp.success);
        assert_eq!(bus_resp.message, "no");
    }
}
