//! Field conversions between typed ROS IDL structs and bus protobuf messages.

pub fn from_ros_string(s: impl ToString) -> String {
    s.to_string()
}

/// Fill a ROS string field. `T` is the distro IDL type (`std::string::String`) or
/// the shim (`rosidl_runtime_rs::String`).
pub fn to_ros_string<T: for<'a> From<&'a str>>(s: impl AsRef<str>) -> T {
    T::from(s.as_ref())
}

/// Reinterpret `Vec<i8>` as `Vec<u8>` without element-wise copies.
/// OccupancyGrid / Costmap `int8[]` and proto `bytes` share the same layout.
pub fn i8_seq_to_bytes(data: Vec<i8>) -> Vec<u8> {
    let mut data = std::mem::ManuallyDrop::new(data);
    // SAFETY: i8 and u8 have identical size/alignment; Vec's heap buffer is interchangeable.
    unsafe { Vec::from_raw_parts(data.as_mut_ptr().cast::<u8>(), data.len(), data.capacity()) }
}

/// Reverse of [`i8_seq_to_bytes`].
pub fn bytes_to_i8_seq(data: Vec<u8>) -> Vec<i8> {
    let mut data = std::mem::ManuallyDrop::new(data);
    // SAFETY: same as [`i8_seq_to_bytes`].
    unsafe { Vec::from_raw_parts(data.as_mut_ptr().cast::<i8>(), data.len(), data.capacity()) }
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

impl FromF64Seq for [f64; 4] {
    fn from_f64_seq(v: Vec<f64>) -> Self {
        let mut a = [0.0; 4];
        for (i, x) in v.into_iter().take(4).enumerate() {
            a[i] = x;
        }
        a
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

impl FromF64Seq for rosidl_runtime_rs::BoundedSequence<f64, 3> {
    fn from_f64_seq(v: Vec<f64>) -> Self {
        let truncated: Vec<f64> = v.into_iter().take(3).collect();
        Self::try_from(truncated).unwrap_or_default()
    }
}

pub trait FromI32Seq: Sized {
    fn from_i32_seq(v: Vec<i32>) -> Self;
}

impl FromI32Seq for Vec<i32> {
    fn from_i32_seq(v: Vec<i32>) -> Self {
        v
    }
}

impl FromI32Seq for Vec<i8> {
    fn from_i32_seq(v: Vec<i32>) -> Self {
        v.into_iter().map(|x| x as i8).collect()
    }
}

impl FromI32Seq for Vec<i16> {
    fn from_i32_seq(v: Vec<i32>) -> Self {
        v.into_iter().map(|x| x as i16).collect()
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

impl FromU32Seq for Vec<u8> {
    fn from_u32_seq(v: Vec<u32>) -> Self {
        v.into_iter().map(|x| x as u8).collect()
    }
}

impl FromU32Seq for Vec<u16> {
    fn from_u32_seq(v: Vec<u32>) -> Self {
        v.into_iter().map(|x| x as u16).collect()
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

pub fn i32_seq(v: impl IntoIterator<Item = impl Into<i32>>) -> Vec<i32> {
    v.into_iter().map(Into::into).collect()
}

pub fn i64_seq(v: impl IntoIterator<Item = i64>) -> Vec<i64> {
    v.into_iter().collect()
}

pub fn u32_seq(v: impl IntoIterator<Item = impl Into<u32>>) -> Vec<u32> {
    v.into_iter().map(Into::into).collect()
}

pub fn string_seq(v: impl IntoIterator<Item = impl ToString>) -> Vec<String> {
    v.into_iter().map(|s| s.to_string()).collect()
}

pub fn ros_string_seq<T: for<'a> From<&'a str>>(v: Vec<String>) -> Vec<T> {
    v.into_iter().map(|s| to_ros_string(s)).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i8_bytes_roundtrip_preserves_layout() {
        let src: Vec<i8> = vec![0, 1, -1, 100, -128, 127];
        let bytes = i8_seq_to_bytes(src.clone());
        assert_eq!(bytes, vec![0, 1, 255, 100, 128, 127]);
        assert_eq!(bytes_to_i8_seq(bytes), src);
    }

    #[test]
    fn i8_bytes_empty() {
        assert!(i8_seq_to_bytes(Vec::new()).is_empty());
        assert!(bytes_to_i8_seq(Vec::new()).is_empty());
    }
}
