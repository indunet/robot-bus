//! Field conversions between typed ROS IDL structs and bus protobuf messages.

use rosidl_runtime_rs::String as RosString;

pub fn from_ros_string(s: impl ToString) -> String {
    s.to_string()
}

pub fn to_ros_string(s: impl AsRef<str>) -> RosString {
    RosString::from(s.as_ref())
}

pub fn i8_seq_to_bytes(data: impl IntoIterator<Item = i8>) -> Vec<u8> {
    data.into_iter().map(|v| v as u8).collect()
}

pub fn bytes_to_i8_seq(data: Vec<u8>) -> Vec<i8> {
    data.into_iter().map(|v| v as i8).collect()
}

pub fn octet_to_bool(v: u8) -> bool {
    v != 0
}

pub fn bool_to_octet(v: bool) -> u8 {
    u8::from(v)
}

pub trait IntoU8Vec {
    fn into_u8_vec(self) -> Vec<u8>;
}

impl IntoU8Vec for Vec<u8> {
    fn into_u8_vec(self) -> Vec<u8> {
        self
    }
}

impl IntoU8Vec for [u8; 16] {
    fn into_u8_vec(self) -> Vec<u8> {
        self.to_vec()
    }
}

/// Build a ROS sequence/array field from protobuf `repeated` / `bytes`.
pub trait FromByteSeq: Sized {
    fn from_byte_seq(v: Vec<u8>) -> Self;
}

impl FromByteSeq for Vec<u8> {
    fn from_byte_seq(v: Vec<u8>) -> Self {
        v
    }
}

impl FromByteSeq for [u8; 16] {
    fn from_byte_seq(v: Vec<u8>) -> Self {
        let mut a = [0u8; 16];
        let n = v.len().min(16);
        a[..n].copy_from_slice(&v[..n]);
        a
    }
}

pub trait FromF64Seq: Sized {
    fn from_f64_seq(v: Vec<f64>) -> Self;
}

impl FromF64Seq for Vec<f64> {
    fn from_f64_seq(v: Vec<f64>) -> Self {
        v
    }
}

impl FromF64Seq for [f64; 9] {
    fn from_f64_seq(v: Vec<f64>) -> Self {
        let mut a = [0.0; 9];
        for (i, x) in v.into_iter().take(9).enumerate() {
            a[i] = x;
        }
        a
    }
}

impl FromF64Seq for [f64; 36] {
    fn from_f64_seq(v: Vec<f64>) -> Self {
        let mut a = [0.0; 36];
        for (i, x) in v.into_iter().take(36).enumerate() {
            a[i] = x;
        }
        a
    }
}

impl FromF64Seq for [f64; 12] {
    fn from_f64_seq(v: Vec<f64>) -> Self {
        let mut a = [0.0; 12];
        for (i, x) in v.into_iter().take(12).enumerate() {
            a[i] = x;
        }
        a
    }
}

pub trait FromU32Seq: Sized {
    fn from_u32_seq(v: Vec<u32>) -> Self;
}

impl FromU32Seq for Vec<u32> {
    fn from_u32_seq(v: Vec<u32>) -> Self {
        v
    }
}

impl FromU32Seq for [u32; 3] {
    fn from_u32_seq(v: Vec<u32>) -> Self {
        let mut a = [0u32; 3];
        for (i, x) in v.into_iter().take(3).enumerate() {
            a[i] = x;
        }
        a
    }
}

pub fn f64_seq(v: impl IntoIterator<Item = f64>) -> Vec<f64> {
    v.into_iter().collect()
}

pub fn f32_seq(v: impl IntoIterator<Item = f32>) -> Vec<f32> {
    v.into_iter().collect()
}

pub fn i32_seq(v: impl IntoIterator<Item = i32>) -> Vec<i32> {
    v.into_iter().collect()
}

pub fn i64_seq(v: impl IntoIterator<Item = i64>) -> Vec<i64> {
    v.into_iter().collect()
}

pub fn u32_seq(v: impl IntoIterator<Item = u32>) -> Vec<u32> {
    v.into_iter().collect()
}

pub fn string_seq(v: impl IntoIterator<Item = impl ToString>) -> Vec<String> {
    v.into_iter().map(|s| s.to_string()).collect()
}

pub fn ros_string_seq(v: Vec<String>) -> Vec<RosString> {
    v.into_iter().map(to_ros_string).collect()
}

pub fn time_to_timestamp(t: ros_env::builtin_interfaces::msg::Time) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: i64::from(t.sec),
        nanos: t.nanosec as i32,
    }
}

pub fn timestamp_to_time(ts: prost_types::Timestamp) -> ros_env::builtin_interfaces::msg::Time {
    ros_env::builtin_interfaces::msg::Time {
        sec: ts.seconds as i32,
        nanosec: ts.nanos as u32,
    }
}

pub fn duration_to_proto(d: ros_env::builtin_interfaces::msg::Duration) -> prost_types::Duration {
    prost_types::Duration {
        seconds: i64::from(d.sec),
        nanos: d.nanosec as i32,
    }
}

pub fn proto_to_duration(d: prost_types::Duration) -> ros_env::builtin_interfaces::msg::Duration {
    ros_env::builtin_interfaces::msg::Duration {
        sec: d.seconds as i32,
        nanosec: d.nanos as u32,
    }
}
