//! Field conversion between `rclrs` [`DynamicMessage`] and robot-bus protobuf.

use rclrs::{
    ArrayValue, ArrayValueMut, DynamicMessage, MessageTypeName, SimpleValue, SimpleValueMut, Value,
    ValueMut,
};

use crate::builtin_interfaces::msg::v1::Time as BusTime;
use crate::geometry_msgs::msg::v1::{Quaternion as BusQuat, Vector3 as BusVec3};
use crate::sensor_msgs::msg::v1::Imu as BusImu;
use crate::std_msgs::msg::v1::{Header as BusHeader, String as BusString};
use crate::BusError;

type Result<T> = std::result::Result<T, BusError>;

fn err(msg: impl Into<String>) -> BusError {
    BusError::Protocol(msg.into())
}

pub fn type_name(kind: super::MsgKind) -> MessageTypeName {
    match kind {
        super::MsgKind::String => MessageTypeName {
            package_name: "std_msgs".into(),
            type_name: "String".into(),
        },
        super::MsgKind::Imu => MessageTypeName {
            package_name: "sensor_msgs".into(),
            type_name: "Imu".into(),
        },
    }
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
    let mut msg = DynamicMessage::new(type_name(super::MsgKind::String))
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
    let mut msg = DynamicMessage::new(type_name(super::MsgKind::Imu))
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
