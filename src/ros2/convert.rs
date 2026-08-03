//! Field conversion between `rclrs` [`DynamicMessage`] and robot-bus protobuf.

use rclrs::{
    ArrayValue, ArrayValueMut, DynamicMessage, MessageTypeName, SimpleValue, SimpleValueMut, Value,
    ValueMut,
};
use prost_types::Timestamp;

use crate::builtin_interfaces::msg::v1::Time as BusTime;
use crate::foxglove_msgs::msg::v1::CompressedVideo as BusCompressedVideo;
use crate::geometry_msgs::msg::v1::{Quaternion as BusQuat, Vector3 as BusVec3};
use crate::sensor_msgs::msg::v1::{Image as BusImage, Imu as BusImu};
use crate::std_msgs::msg::v1::{Header as BusHeader, String as BusString};
use crate::BusError;

use super::codec::{read_bool_or_u8, read_byte_sequence, write_bool_as_u8, write_byte_sequence};

type Result<T> = std::result::Result<T, BusError>;

fn err(msg: impl Into<String>) -> BusError {
    BusError::Protocol(msg.into())
}

fn ros_type(full: &str) -> Result<MessageTypeName> {
    MessageTypeName::try_from(full).map_err(|e| err(format!("invalid ROS type {full:?}: {e}")))
}

fn read_string(msg: &DynamicMessage, field: &str) -> Result<String> {
    match msg.get(field) {
        Some(Value::Simple(SimpleValue::String(s))) => Ok(s.to_string()),
        other => Err(err(format!(
            "expected string field `{field}`, got {other:?}"
        ))),
    }
}

fn write_string(msg: &mut DynamicMessage, field: &str, value: &str) -> Result<()> {
    match msg.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::String(s))) => {
            *s = value.into();
            Ok(())
        }
        other => Err(err(format!(
            "expected mut string field `{field}`, got {other:?}"
        ))),
    }
}

fn read_f64(view: &rclrs::DynamicMessageView<'_>, field: &str) -> Result<f64> {
    match view.get(field) {
        Some(Value::Simple(SimpleValue::Double(v))) => Ok(*v),
        other => Err(err(format!(
            "expected f64 field `{field}`, got {other:?}"
        ))),
    }
}

fn write_f64(view: &mut rclrs::DynamicMessageViewMut<'_>, field: &str, value: f64) -> Result<()> {
    match view.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::Double(v))) => {
            *v = value;
            Ok(())
        }
        other => Err(err(format!(
            "expected mut f64 field `{field}`, got {other:?}"
        ))),
    }
}

fn read_i32(view: &rclrs::DynamicMessageView<'_>, field: &str) -> Result<i32> {
    match view.get(field) {
        Some(Value::Simple(SimpleValue::Int32(v))) => Ok(*v),
        other => Err(err(format!(
            "expected i32 field `{field}`, got {other:?}"
        ))),
    }
}

fn write_i32(view: &mut rclrs::DynamicMessageViewMut<'_>, field: &str, value: i32) -> Result<()> {
    match view.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::Int32(v))) => {
            *v = value;
            Ok(())
        }
        other => Err(err(format!(
            "expected mut i32 field `{field}`, got {other:?}"
        ))),
    }
}

fn read_u32(view: &rclrs::DynamicMessageView<'_>, field: &str) -> Result<u32> {
    match view.get(field) {
        Some(Value::Simple(SimpleValue::Uint32(v))) => Ok(*v),
        other => Err(err(format!(
            "expected u32 field `{field}`, got {other:?}"
        ))),
    }
}

fn write_u32(view: &mut rclrs::DynamicMessageViewMut<'_>, field: &str, value: u32) -> Result<()> {
    match view.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::Uint32(v))) => {
            *v = value;
            Ok(())
        }
        other => Err(err(format!(
            "expected mut u32 field `{field}`, got {other:?}"
        ))),
    }
}

fn read_cov9(msg: &DynamicMessage, field: &str) -> Result<Vec<f64>> {
    match msg.get(field) {
        Some(Value::Array(ArrayValue::DoubleArray(arr))) => Ok(arr.to_vec()),
        other => Err(err(format!(
            "expected double[9] field `{field}`, got {other:?}"
        ))),
    }
}

fn write_cov9(msg: &mut DynamicMessage, field: &str, values: &[f64]) -> Result<()> {
    match msg.get_mut(field) {
        Some(ValueMut::Array(ArrayValueMut::DoubleArray(arr))) => {
            let n = arr.len().min(values.len()).min(9);
            for i in 0..n {
                arr[i] = values[i];
            }
            Ok(())
        }
        other => Err(err(format!(
            "expected mut double[9] field `{field}`, got {other:?}"
        ))),
    }
}

fn nested_view<'a>(
    msg: &'a DynamicMessage,
    field: &str,
) -> Result<rclrs::DynamicMessageView<'a>> {
    match msg.get(field) {
        Some(Value::Simple(SimpleValue::Message(view))) => Ok(view),
        other => Err(err(format!(
            "expected nested message `{field}`, got {other:?}"
        ))),
    }
}

fn with_nested_mut<R>(
    msg: &mut DynamicMessage,
    field: &str,
    f: impl FnOnce(&mut rclrs::DynamicMessageViewMut<'_>) -> Result<R>,
) -> Result<R> {
    match msg.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::Message(mut view))) => f(&mut view),
        other => Err(err(format!(
            "expected mut nested message `{field}`, got {other:?}"
        ))),
    }
}

fn header_from_view(view: &rclrs::DynamicMessageView<'_>) -> Result<BusHeader> {
    let frame_id = match view.get("frame_id") {
        Some(Value::Simple(SimpleValue::String(s))) => s.to_string(),
        other => {
            return Err(err(format!(
                "expected header.frame_id string, got {other:?}"
            )))
        }
    };
    let mut header = BusHeader {
        frame_id,
        stamp: None,
    };
    if let Some(Value::Simple(SimpleValue::Message(stamp))) = view.get("stamp") {
        header.stamp = Some(BusTime {
            sec: read_i32(&stamp, "sec")?,
            nanosec: read_u32(&stamp, "nanosec")?,
        });
    }
    Ok(header)
}

fn write_header(msg: &mut DynamicMessage, header: &BusHeader) -> Result<()> {
    with_nested_mut(msg, "header", |view| {
        match view.get_mut("frame_id") {
            Some(ValueMut::Simple(SimpleValueMut::String(s))) => *s = header.frame_id.as_str().into(),
            other => {
                return Err(err(format!(
                    "expected mut header.frame_id, got {other:?}"
                )))
            }
        }
        if let Some(stamp) = &header.stamp {
            match view.get_mut("stamp") {
                Some(ValueMut::Simple(SimpleValueMut::Message(mut stamp_view))) => {
                    write_i32(&mut stamp_view, "sec", stamp.sec)?;
                    write_u32(&mut stamp_view, "nanosec", stamp.nanosec)?;
                }
                other => {
                    return Err(err(format!("expected mut header.stamp, got {other:?}")))
                }
            }
        }
        Ok(())
    })
}

fn vec3_from_view(view: &rclrs::DynamicMessageView<'_>) -> Result<BusVec3> {
    Ok(BusVec3 {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
        z: read_f64(view, "z")?,
    })
}

fn write_vec3(parent: &mut DynamicMessage, field: &str, v: &BusVec3) -> Result<()> {
    with_nested_mut(parent, field, |view| {
        write_f64(view, "x", v.x)?;
        write_f64(view, "y", v.y)?;
        write_f64(view, "z", v.z)?;
        Ok(())
    })
}

fn quat_from_view(view: &rclrs::DynamicMessageView<'_>) -> Result<BusQuat> {
    Ok(BusQuat {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
        z: read_f64(view, "z")?,
        w: read_f64(view, "w")?,
    })
}

fn write_quat(parent: &mut DynamicMessage, field: &str, q: &BusQuat) -> Result<()> {
    with_nested_mut(parent, field, |view| {
        write_f64(view, "x", q.x)?;
        write_f64(view, "y", q.y)?;
        write_f64(view, "z", q.z)?;
        write_f64(view, "w", q.w)?;
        Ok(())
    })
}

pub fn string_dyn_to_bus(msg: &DynamicMessage) -> Result<BusString> {
    Ok(BusString {
        data: read_string(msg, "data")?,
    })
}

pub fn string_bus_to_dyn(bus: &BusString) -> Result<DynamicMessage> {
    let mut msg = DynamicMessage::new(ros_type("std_msgs/msg/String")?)
        .map_err(|e| err(format!("create std_msgs/String: {e}")))?;
    write_string(&mut msg, "data", &bus.data)?;
    Ok(msg)
}

pub fn imu_dyn_to_bus(msg: &DynamicMessage) -> Result<BusImu> {
    let header = header_from_view(&nested_view(msg, "header")?)?;
    let orientation = quat_from_view(&nested_view(msg, "orientation")?)?;
    let angular_velocity = vec3_from_view(&nested_view(msg, "angular_velocity")?)?;
    let linear_acceleration = vec3_from_view(&nested_view(msg, "linear_acceleration")?)?;
    Ok(BusImu {
        header: Some(header),
        orientation: Some(orientation),
        orientation_covariance: read_cov9(msg, "orientation_covariance")?,
        angular_velocity: Some(angular_velocity),
        angular_velocity_covariance: read_cov9(msg, "angular_velocity_covariance")?,
        linear_acceleration: Some(linear_acceleration),
        linear_acceleration_covariance: read_cov9(msg, "linear_acceleration_covariance")?,
    })
}

pub fn imu_bus_to_dyn(bus: &BusImu) -> Result<DynamicMessage> {
    let mut msg = DynamicMessage::new(ros_type("sensor_msgs/msg/Imu")?)
        .map_err(|e| err(format!("create sensor_msgs/Imu: {e}")))?;
    if let Some(header) = &bus.header {
        write_header(&mut msg, header)?;
    }
    if let Some(q) = &bus.orientation {
        write_quat(&mut msg, "orientation", q)?;
    }
    write_cov9(&mut msg, "orientation_covariance", &bus.orientation_covariance)?;
    if let Some(v) = &bus.angular_velocity {
        write_vec3(&mut msg, "angular_velocity", v)?;
    }
    write_cov9(
        &mut msg,
        "angular_velocity_covariance",
        &bus.angular_velocity_covariance,
    )?;
    if let Some(v) = &bus.linear_acceleration {
        write_vec3(&mut msg, "linear_acceleration", v)?;
    }
    write_cov9(
        &mut msg,
        "linear_acceleration_covariance",
        &bus.linear_acceleration_covariance,
    )?;
    Ok(msg)
}

fn read_u32_field(msg: &DynamicMessage, field: &str) -> Result<u32> {
    match msg.get(field) {
        Some(Value::Simple(SimpleValue::Uint32(v))) => Ok(*v),
        other => Err(err(format!(
            "expected u32 field `{field}`, got {other:?}"
        ))),
    }
}

fn write_u32_field(msg: &mut DynamicMessage, field: &str, value: u32) -> Result<()> {
    match msg.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::Uint32(v))) => {
            *v = value;
            Ok(())
        }
        other => Err(err(format!(
            "expected mut u32 field `{field}`, got {other:?}"
        ))),
    }
}

fn timestamp_from_time_view(view: &rclrs::DynamicMessageView<'_>) -> Result<Timestamp> {
    Ok(Timestamp {
        seconds: i64::from(read_i32(view, "sec")?),
        nanos: read_u32(view, "nanosec")? as i32,
    })
}

fn write_time_field(msg: &mut DynamicMessage, field: &str, ts: &Timestamp) -> Result<()> {
    with_nested_mut(msg, field, |view| {
        write_i32(view, "sec", ts.seconds as i32)?;
        write_u32(view, "nanosec", ts.nanos as u32)?;
        Ok(())
    })
}

pub fn image_dyn_to_bus(msg: &DynamicMessage) -> Result<BusImage> {
    let header = header_from_view(&nested_view(msg, "header")?)?;
    Ok(BusImage {
        header: Some(header),
        height: read_u32_field(msg, "height")?,
        width: read_u32_field(msg, "width")?,
        encoding: read_string(msg, "encoding")?,
        is_bigendian: read_bool_or_u8(msg, "is_bigendian")?,
        step: read_u32_field(msg, "step")?,
        data: read_byte_sequence(msg, "data")?,
    })
}

pub fn image_bus_to_dyn(bus: &BusImage) -> Result<DynamicMessage> {
    let mut msg = DynamicMessage::new(ros_type("sensor_msgs/msg/Image")?)
        .map_err(|e| err(format!("create sensor_msgs/Image: {e}")))?;
    if let Some(header) = &bus.header {
        write_header(&mut msg, header)?;
    }
    write_u32_field(&mut msg, "height", bus.height)?;
    write_u32_field(&mut msg, "width", bus.width)?;
    write_string(&mut msg, "encoding", &bus.encoding)?;
    write_bool_as_u8(&mut msg, "is_bigendian", bus.is_bigendian)?;
    write_u32_field(&mut msg, "step", bus.step)?;
    write_byte_sequence(&mut msg, "data", &bus.data)?;
    Ok(msg)
}

pub fn compressed_video_dyn_to_bus(msg: &DynamicMessage) -> Result<BusCompressedVideo> {
    let timestamp = match msg.get("timestamp") {
        Some(Value::Simple(SimpleValue::Message(view))) => Some(timestamp_from_time_view(&view)?),
        None => None,
        other => {
            return Err(err(format!(
                "expected nested timestamp message, got {other:?}"
            )))
        }
    };
    Ok(BusCompressedVideo {
        timestamp,
        frame_id: read_string(msg, "frame_id")?,
        data: read_byte_sequence(msg, "data")?,
        format: read_string(msg, "format")?,
    })
}

pub fn compressed_video_bus_to_dyn(bus: &BusCompressedVideo) -> Result<DynamicMessage> {
    let mut msg = DynamicMessage::new(ros_type("foxglove_msgs/msg/CompressedVideo")?)
        .map_err(|e| err(format!("create foxglove_msgs/CompressedVideo: {e}")))?;
    if let Some(ts) = &bus.timestamp {
        write_time_field(&mut msg, "timestamp", ts)?;
    }
    write_string(&mut msg, "frame_id", &bus.frame_id)?;
    write_byte_sequence(&mut msg, "data", &bus.data)?;
    write_string(&mut msg, "format", &bus.format)?;
    Ok(msg)
}

// --- std_srvs typed conversions (rclrs vendor ↔ bus prost) ---

use crate::std_srvs::srv::v1::{
    SetBoolRequest as BusSetBoolRequest, SetBoolResponse as BusSetBoolResponse,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};
use super::vendor::std_srvs::srv as ros_srv;

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

// --- Fibonacci action conversions (rclrs vendor ↔ bus prost) ---

use crate::action::v1::{
    FibonacciFeedback as BusFibonacciFeedback, FibonacciGoal as BusFibonacciGoal,
    FibonacciResult as BusFibonacciResult,
};
use rclrs::vendor::example_interfaces::action as ros_act;

pub fn fibonacci_ros_goal_to_bus(goal: &ros_act::Fibonacci_Goal) -> BusFibonacciGoal {
    BusFibonacciGoal { order: goal.order }
}

pub fn fibonacci_bus_goal_to_ros(goal: &BusFibonacciGoal) -> ros_act::Fibonacci_Goal {
    ros_act::Fibonacci_Goal { order: goal.order }
}

pub fn fibonacci_ros_feedback_to_bus(fb: &ros_act::Fibonacci_Feedback) -> BusFibonacciFeedback {
    BusFibonacciFeedback {
        sequence: fb.sequence.clone(),
    }
}

pub fn fibonacci_bus_feedback_to_ros(fb: &BusFibonacciFeedback) -> ros_act::Fibonacci_Feedback {
    ros_act::Fibonacci_Feedback {
        sequence: fb.sequence.clone(),
    }
}

pub fn fibonacci_ros_result_to_bus(res: &ros_act::Fibonacci_Result) -> BusFibonacciResult {
    BusFibonacciResult {
        sequence: res.sequence.clone(),
    }
}

pub fn fibonacci_bus_result_to_ros(res: &BusFibonacciResult) -> ros_act::Fibonacci_Result {
    ros_act::Fibonacci_Result {
        sequence: res.sequence.clone(),
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

#[cfg(test)]
mod action_convert_tests {
    use super::*;

    #[test]
    fn fibonacci_goal_feedback_result_roundtrip() {
        let ros_goal = ros_act::Fibonacci_Goal { order: 5 };
        let bus_goal = fibonacci_ros_goal_to_bus(&ros_goal);
        assert_eq!(bus_goal.order, 5);
        assert_eq!(fibonacci_bus_goal_to_ros(&bus_goal).order, 5);

        let ros_fb = ros_act::Fibonacci_Feedback {
            sequence: vec![0, 1, 1, 2],
        };
        let bus_fb = fibonacci_ros_feedback_to_bus(&ros_fb);
        assert_eq!(bus_fb.sequence, vec![0, 1, 1, 2]);
        assert_eq!(
            fibonacci_bus_feedback_to_ros(&bus_fb).sequence,
            vec![0, 1, 1, 2]
        );

        let ros_res = ros_act::Fibonacci_Result {
            sequence: vec![0, 1, 1, 2, 3],
        };
        let bus_res = fibonacci_ros_result_to_bus(&ros_res);
        assert_eq!(bus_res.sequence, vec![0, 1, 1, 2, 3]);
        assert_eq!(
            fibonacci_bus_result_to_ros(&bus_res).sequence,
            vec![0, 1, 1, 2, 3]
        );
    }
}
