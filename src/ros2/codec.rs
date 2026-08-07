//! Topic type registry: map ROS type strings → [`TopicCodec`] converters.
//!
//! Add a new bridged topic type by implementing [`TopicCodec`] and registering it in
//! [`BUILTIN_CODECS`]. Builder / YAML / wire paths do not need per-type match arms.

use std::collections::HashMap;
use std::sync::LazyLock;

use prost::Message as ProstMessage;
use rclrs::{
    DynamicMessage, MessageTypeName, SequenceValue, SequenceValueMut, SimpleValue, SimpleValueMut,
    Value, ValueMut,
};
use rosidl_runtime_rs::Sequence;

use crate::BusError;
use crate::foxglove_msgs::msg::v1::CompressedVideo as BusCompressedVideo;
use crate::sensor_msgs::msg::v1::{Image as BusImage, Imu as BusImu};
use crate::std_msgs::msg::v1::String as BusString;

use super::convert;

type Result<T> = std::result::Result<T, BusError>;

fn err(msg: impl Into<String>) -> BusError {
    BusError::Protocol(msg.into())
}

/// Bidirectional converter between ROS [`DynamicMessage`] and bus protobuf bytes.
pub trait TopicCodec: Send + Sync {
    /// Full ROS type name, e.g. `sensor_msgs/msg/Image`.
    fn type_name(&self) -> &'static str;

    fn ros_type(&self) -> MessageTypeName {
        MessageTypeName::try_from(self.type_name())
            .expect("TopicCodec::type_name must be package/msg/Type")
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>>;
    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage>;
}

static BUILTIN_CODECS: LazyLock<HashMap<&'static str, &'static dyn TopicCodec>> =
    LazyLock::new(|| {
        let codecs: &[&'static dyn TopicCodec] =
            &[&StringCodec, &ImuCodec, &ImageCodec, &CompressedVideoCodec];
        let mut map = HashMap::with_capacity(codecs.len());
        for c in codecs {
            map.insert(c.type_name(), *c);
        }
        map
    });

/// Look up a registered topic codec by ROS type string.
pub fn lookup_topic_codec(type_name: &str) -> Result<&'static dyn TopicCodec> {
    BUILTIN_CODECS.get(type_name).copied().ok_or_else(|| {
        let mut supported: Vec<&'static str> = BUILTIN_CODECS.keys().copied().collect();
        supported.sort_unstable();
        BusError::Protocol(format!(
            "unsupported ros2 bridge topic type {type_name:?}; registered: {}",
            supported.join(", ")
        ))
    })
}

/// Sorted list of registered topic type names (for docs / errors).
pub fn registered_topic_types() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = BUILTIN_CODECS.keys().copied().collect();
    v.sort_unstable();
    v
}

// --- String ---

struct StringCodec;

impl TopicCodec for StringCodec {
    fn type_name(&self) -> &'static str {
        "std_msgs/msg/String"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(convert::string_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = BusString::decode(payload).map_err(|e| err(format!("decode String: {e}")))?;
        convert::string_bus_to_dyn(&bus)
    }
}

// --- Imu ---

struct ImuCodec;

impl TopicCodec for ImuCodec {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Imu"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(convert::imu_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = BusImu::decode(payload).map_err(|e| err(format!("decode Imu: {e}")))?;
        convert::imu_bus_to_dyn(&bus)
    }
}

// --- Image ---

struct ImageCodec;

impl TopicCodec for ImageCodec {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Image"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(convert::image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = BusImage::decode(payload).map_err(|e| err(format!("decode Image: {e}")))?;
        convert::image_bus_to_dyn(&bus)
    }
}

// --- CompressedVideo ---

struct CompressedVideoCodec;

impl TopicCodec for CompressedVideoCodec {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedVideo"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(convert::compressed_video_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = BusCompressedVideo::decode(payload)
            .map_err(|e| err(format!("decode CompressedVideo: {e}")))?;
        convert::compressed_video_bus_to_dyn(&bus)
    }
}

/// Helpers used by convert + tests for reading/writing `uint8[]` / `octet[]` fields.
pub(crate) fn read_byte_sequence(msg: &DynamicMessage, field: &str) -> Result<Vec<u8>> {
    match msg.get(field) {
        Some(Value::Sequence(SequenceValue::Uint8Sequence(seq)))
        | Some(Value::Sequence(SequenceValue::OctetSequence(seq)))
        | Some(Value::Sequence(SequenceValue::CharSequence(seq))) => Ok(seq.as_slice().to_vec()),
        other => Err(err(format!(
            "expected uint8[] field `{field}`, got {other:?}"
        ))),
    }
}

pub(crate) fn write_byte_sequence(
    msg: &mut DynamicMessage,
    field: &str,
    data: &[u8],
) -> Result<()> {
    match msg.get_mut(field) {
        Some(ValueMut::Sequence(SequenceValueMut::Uint8Sequence(seq)))
        | Some(ValueMut::Sequence(SequenceValueMut::OctetSequence(seq)))
        | Some(ValueMut::Sequence(SequenceValueMut::CharSequence(seq))) => {
            *seq = Sequence::from(data);
            Ok(())
        }
        other => Err(err(format!(
            "expected mut uint8[] field `{field}`, got {other:?}"
        ))),
    }
}

pub(crate) fn read_bool_or_u8(msg: &DynamicMessage, field: &str) -> Result<bool> {
    match msg.get(field) {
        Some(Value::Simple(SimpleValue::Boolean(v))) => Ok(*v),
        Some(Value::Simple(SimpleValue::Uint8(v))) | Some(Value::Simple(SimpleValue::Octet(v))) => {
            Ok(*v != 0)
        }
        other => Err(err(format!(
            "expected bool/uint8 field `{field}`, got {other:?}"
        ))),
    }
}

pub(crate) fn write_bool_as_u8(msg: &mut DynamicMessage, field: &str, value: bool) -> Result<()> {
    let byte = u8::from(value);
    match msg.get_mut(field) {
        Some(ValueMut::Simple(SimpleValueMut::Boolean(v))) => {
            *v = value;
            Ok(())
        }
        Some(ValueMut::Simple(SimpleValueMut::Uint8(v)))
        | Some(ValueMut::Simple(SimpleValueMut::Octet(v))) => {
            *v = byte;
            Ok(())
        }
        other => Err(err(format!(
            "expected mut bool/uint8 field `{field}`, got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::Timestamp;

    #[test]
    fn registry_lists_four_builtins() {
        let types = registered_topic_types();
        assert_eq!(
            types,
            vec![
                "foxglove_msgs/msg/CompressedVideo",
                "sensor_msgs/msg/Image",
                "sensor_msgs/msg/Imu",
                "std_msgs/msg/String",
            ]
        );
    }

    #[test]
    fn lookup_unknown_lists_supported() {
        let Err(e) = lookup_topic_codec("std_msgs/msg/Empty") else {
            panic!("expected lookup failure");
        };
        let err = e.to_string();
        assert!(err.contains("unsupported"));
        assert!(err.contains("sensor_msgs/msg/Image"));
        assert!(err.contains("foxglove_msgs/msg/CompressedVideo"));
    }

    #[test]
    fn lookup_known_types() {
        for t in [
            "std_msgs/msg/String",
            "sensor_msgs/msg/Imu",
            "sensor_msgs/msg/Image",
            "foxglove_msgs/msg/CompressedVideo",
        ] {
            assert_eq!(lookup_topic_codec(t).unwrap().type_name(), t);
        }
    }

    #[test]
    fn image_roundtrip_when_typesupport_available() {
        let codec = lookup_topic_codec("sensor_msgs/msg/Image").unwrap();
        let bus = BusImage {
            header: Some(crate::std_msgs::msg::v1::Header {
                frame_id: "cam".into(),
                stamp: Some(crate::builtin_interfaces::msg::v1::Time { sec: 1, nanosec: 2 }),
            }),
            height: 2,
            width: 2,
            encoding: "rgb8".into(),
            is_bigendian: false,
            step: 6,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        };
        let Ok(dyn_msg) = codec.bus_to_ros(&bus.encode_to_vec()) else {
            // No ROS type support in this environment (e.g. ros2-shim).
            return;
        };
        let back = BusImage::decode(codec.ros_to_bus(&dyn_msg).unwrap().as_slice()).unwrap();
        assert_eq!(back.height, 2);
        assert_eq!(back.width, 2);
        assert_eq!(back.encoding, "rgb8");
        assert!(!back.is_bigendian);
        assert_eq!(back.step, 6);
        assert_eq!(back.data, bus.data);
        assert_eq!(back.header.as_ref().unwrap().frame_id, "cam");
    }

    #[test]
    fn compressed_video_roundtrip_when_typesupport_available() {
        let codec = lookup_topic_codec("foxglove_msgs/msg/CompressedVideo").unwrap();
        let bus = BusCompressedVideo {
            timestamp: Some(Timestamp {
                seconds: 10,
                nanos: 20,
            }),
            frame_id: "cam".into(),
            data: vec![0x00, 0x00, 0x00, 0x01, 0x67],
            format: "h264".into(),
        };
        let Ok(dyn_msg) = codec.bus_to_ros(&bus.encode_to_vec()) else {
            return;
        };
        let back =
            BusCompressedVideo::decode(codec.ros_to_bus(&dyn_msg).unwrap().as_slice()).unwrap();
        assert_eq!(back.frame_id, "cam");
        assert_eq!(back.format, "h264");
        assert_eq!(back.data, bus.data);
        let ts = back.timestamp.unwrap();
        assert_eq!(ts.seconds, 10);
        assert_eq!(ts.nanos, 20);
    }
}
