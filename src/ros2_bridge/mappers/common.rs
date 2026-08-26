//! Shared DynamicMessage field accessors for topic mappers.


use prost_types::{Duration as ProstDuration, Timestamp};
use rclrs::{
    ArrayValue, ArrayValueMut, BoundedSequenceValue, BoundedSequenceValueMut, DynamicMessage,
    DynamicMessageView, DynamicMessageViewMut, MessageTypeName, SequenceValue, SequenceValueMut,
    SimpleValue, SimpleValueMut, Value, ValueMut,
};
use rosidl_runtime_rs::Sequence;

use crate::BusError;


pub type Result<T> = std::result::Result<T, BusError>;

pub fn err(msg: impl Into<String>) -> BusError {
    BusError::Protocol(msg.into())
}

pub fn ros_type(full: &str) -> Result<MessageTypeName> {
    MessageTypeName::try_from(full).map_err(|e| err(format!("invalid ROS type {full:?}: {e}")))
}

/// Allocate an empty [`DynamicMessage`]; fails when the type support is not installed.
pub fn new_message(full: &str) -> Result<DynamicMessage> {
    DynamicMessage::new(ros_type(full)?).map_err(|e| err(format!("create {full}: {e}")))
}

// --- scalars ---

fn simple_to_i64(value: &SimpleValue<'_>) -> Option<i64> {
    Some(match value {
        SimpleValue::Boolean(v) => i64::from(**v),
        SimpleValue::Char(v) | SimpleValue::Octet(v) | SimpleValue::Uint8(v) => i64::from(**v),
        SimpleValue::Int8(v) => i64::from(**v),
        SimpleValue::WChar(v) | SimpleValue::Uint16(v) => i64::from(**v),
        SimpleValue::Int16(v) => i64::from(**v),
        SimpleValue::Uint32(v) => i64::from(**v),
        SimpleValue::Int32(v) => i64::from(**v),
        SimpleValue::Uint64(v) => **v as i64,
        SimpleValue::Int64(v) => **v,
        SimpleValue::Float(v) => **v as i64,
        SimpleValue::Double(v) => **v as i64,
        _ => return None,
    })
}

fn simple_to_f64(value: &SimpleValue<'_>) -> Option<f64> {
    Some(match value {
        SimpleValue::Float(v) => f64::from(**v),
        SimpleValue::Double(v) => **v,
        other => simple_to_i64(other)? as f64,
    })
}

fn simple_set_i64(slot: SimpleValueMut<'_>, value: i64) -> bool {
    match slot {
        SimpleValueMut::Boolean(v) => *v = value != 0,
        SimpleValueMut::Char(v) | SimpleValueMut::Octet(v) | SimpleValueMut::Uint8(v) => {
            *v = value as u8
        }
        SimpleValueMut::Int8(v) => *v = value as i8,
        SimpleValueMut::WChar(v) | SimpleValueMut::Uint16(v) => *v = value as u16,
        SimpleValueMut::Int16(v) => *v = value as i16,
        SimpleValueMut::Uint32(v) => *v = value as u32,
        SimpleValueMut::Int32(v) => *v = value as i32,
        SimpleValueMut::Uint64(v) => *v = value as u64,
        SimpleValueMut::Int64(v) => *v = value,
        SimpleValueMut::Float(v) => *v = value as f32,
        SimpleValueMut::Double(v) => *v = value as f64,
        _ => return false,
    }
    true
}

fn simple_set_f64(slot: SimpleValueMut<'_>, value: f64) -> bool {
    match slot {
        SimpleValueMut::Float(v) => *v = value as f32,
        SimpleValueMut::Double(v) => *v = value,
        other => return simple_set_i64(other, value as i64),
    }
    true
}

/// True when the ROS message actually declares `field`.
pub fn has_field(view: &DynamicMessageView<'_>, field: &str) -> bool {
    view.get(field).is_some()
}

pub fn read_i64(view: &DynamicMessageView<'_>, field: &str) -> Result<i64> {
    match view.get(field) {
        None => Ok(0),
        Some(Value::Simple(v)) => {
            simple_to_i64(&v).ok_or_else(|| err(format!("field `{field}` is not a number: {v:?}")))
        }
        Some(other) => Err(err(format!(
            "expected scalar field `{field}`, got {other:?}"
        ))),
    }
}

pub fn read_f64(view: &DynamicMessageView<'_>, field: &str) -> Result<f64> {
    match view.get(field) {
        None => Ok(0.0),
        Some(Value::Simple(v)) => {
            simple_to_f64(&v).ok_or_else(|| err(format!("field `{field}` is not a number: {v:?}")))
        }
        Some(other) => Err(err(format!(
            "expected scalar field `{field}`, got {other:?}"
        ))),
    }
}

pub fn write_i64(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    value: i64,
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(ValueMut::Simple(v)) => {
            if simple_set_i64(v, value) {
                Ok(())
            } else {
                Err(err(format!("field `{field}` is not a writable number")))
            }
        }
        Some(other) => Err(err(format!(
            "expected mut scalar field `{field}`, got {other:?}"
        ))),
    }
}

pub fn write_f64(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    value: f64,
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(ValueMut::Simple(v)) => {
            if simple_set_f64(v, value) {
                Ok(())
            } else {
                Err(err(format!("field `{field}` is not a writable number")))
            }
        }
        Some(other) => Err(err(format!(
            "expected mut scalar field `{field}`, got {other:?}"
        ))),
    }
}

pub fn read_f32(view: &DynamicMessageView<'_>, field: &str) -> Result<f32> {
    Ok(read_f64(view, field)? as f32)
}

pub fn read_i32(view: &DynamicMessageView<'_>, field: &str) -> Result<i32> {
    Ok(read_i64(view, field)? as i32)
}

pub fn read_u32(view: &DynamicMessageView<'_>, field: &str) -> Result<u32> {
    Ok(read_i64(view, field)? as u32)
}

pub fn read_u64(view: &DynamicMessageView<'_>, field: &str) -> Result<u64> {
    Ok(read_i64(view, field)? as u64)
}

pub fn read_bool(view: &DynamicMessageView<'_>, field: &str) -> Result<bool> {
    Ok(read_i64(view, field)? != 0)
}

pub fn write_f32(view: &mut DynamicMessageViewMut<'_>, field: &str, v: f32) -> Result<()> {
    write_f64(view, field, f64::from(v))
}

pub fn write_i32(view: &mut DynamicMessageViewMut<'_>, field: &str, v: i32) -> Result<()> {
    write_i64(view, field, i64::from(v))
}

pub fn write_u32(view: &mut DynamicMessageViewMut<'_>, field: &str, v: u32) -> Result<()> {
    write_i64(view, field, i64::from(v))
}

pub fn write_u64(view: &mut DynamicMessageViewMut<'_>, field: &str, v: u64) -> Result<()> {
    write_i64(view, field, v as i64)
}

pub fn write_bool(view: &mut DynamicMessageViewMut<'_>, field: &str, v: bool) -> Result<()> {
    write_i64(view, field, i64::from(v))
}

/// `None` when the ROS message has no such field (protobuf `optional double`).
pub fn read_f64_opt(view: &DynamicMessageView<'_>, field: &str) -> Result<Option<f64>> {
    if has_field(view, field) {
        Ok(Some(read_f64(view, field)?))
    } else {
        Ok(None)
    }
}

// --- strings ---

pub fn read_string(view: &DynamicMessageView<'_>, field: &str) -> Result<String> {
    match view.get(field) {
        None => Ok(String::new()),
        Some(Value::Simple(SimpleValue::String(s))) => Ok(s.to_string()),
        Some(Value::Simple(SimpleValue::BoundedString(s))) => Ok(s.to_string()),
        Some(Value::Simple(SimpleValue::WString(s))) => Ok(s.to_string()),
        Some(Value::Simple(SimpleValue::BoundedWString(s))) => Ok(s.to_string()),
        Some(other) => Err(err(format!(
            "expected string field `{field}`, got {other:?}"
        ))),
    }
}

pub fn write_string(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    value: &str,
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(ValueMut::Simple(SimpleValueMut::String(s))) => {
            *s = value.into();
            Ok(())
        }
        Some(ValueMut::Simple(SimpleValueMut::BoundedString(mut s))) => {
            let _ = s.try_assign(value);
            Ok(())
        }
        Some(ValueMut::Simple(SimpleValueMut::WString(s))) => {
            *s = value.into();
            Ok(())
        }
        Some(ValueMut::Simple(SimpleValueMut::BoundedWString(mut s))) => {
            let _ = s.try_assign(value);
            Ok(())
        }
        Some(other) => Err(err(format!(
            "expected mut string field `{field}`, got {other:?}"
        ))),
    }
}

// --- nested messages ---

pub fn nested_view<'msg>(
    view: &DynamicMessageView<'msg>,
    field: &str,
) -> Result<Option<DynamicMessageView<'msg>>> {
    match view.get(field) {
        None => Ok(None),
        Some(Value::Simple(SimpleValue::Message(v))) => Ok(Some(v)),
        Some(other) => Err(err(format!(
            "expected nested message `{field}`, got {other:?}"
        ))),
    }
}

pub fn with_nested_mut(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    f: impl FnOnce(&mut DynamicMessageViewMut<'_>) -> Result<()>,
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(ValueMut::Simple(SimpleValueMut::Message(mut v))) => f(&mut v),
        Some(other) => Err(err(format!(
            "expected mut nested message `{field}`, got {other:?}"
        ))),
    }
}

pub fn read_message_seq<T>(
    view: &DynamicMessageView<'_>,
    field: &str,
    f: impl Fn(&DynamicMessageView<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let elements: Vec<T> = match view.get(field) {
        None => Vec::new(),
        Some(Value::Sequence(SequenceValue::MessageSequence(seq))) => {
            seq.as_slice().iter().map(&f).collect::<Result<Vec<T>>>()?
        }
        Some(Value::Array(ArrayValue::MessageArray(items))) => {
            items.iter().map(&f).collect::<Result<Vec<T>>>()?
        }
        Some(Value::BoundedSequence(BoundedSequenceValue::MessageBoundedSequence(seq))) => {
            seq.iter().map(&f).collect::<Result<Vec<T>>>()?
        }
        Some(other) => {
            return Err(err(format!(
                "expected message sequence `{field}`, got {other:?}"
            )));
        }
    };
    Ok(elements)
}

pub fn write_message_seq<T>(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    items: &[T],
    f: impl Fn(&mut DynamicMessageViewMut<'_>, &T) -> Result<()>,
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(ValueMut::Sequence(SequenceValueMut::MessageSequence(mut seq))) => {
            seq.reset(items.len());
            let slots = seq.as_mut_slice();
            for (slot, item) in slots.iter_mut().zip(items) {
                f(slot, item)?;
            }
            Ok(())
        }
        Some(ValueMut::Array(ArrayValueMut::MessageArray(mut slots))) => {
            for (slot, item) in slots.iter_mut().zip(items) {
                f(slot, item)?;
            }
            Ok(())
        }
        Some(ValueMut::BoundedSequence(BoundedSequenceValueMut::MessageBoundedSequence(
            mut seq,
        ))) => {
            let len = items.len().min(seq.upper_bound());
            let _ = seq.try_reset(len);
            let slots = seq.as_mut_slice();
            for (slot, item) in slots.iter_mut().zip(items) {
                f(slot, item)?;
            }
            Ok(())
        }
        Some(other) => Err(err(format!(
            "expected mut message sequence `{field}`, got {other:?}"
        ))),
    }
}

// --- numeric sequences (ROS `T[]`, `T[N]` and `T[<=N]` all map to protobuf `repeated`) ---

macro_rules! numeric_seq_to_vec {
    ($value:expr, $out:ty) => {
        match $value {
            Value::Sequence(seq) => match seq {
                SequenceValue::FloatSequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::DoubleSequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::CharSequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::WCharSequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::OctetSequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Uint8Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Int8Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Uint16Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Int16Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Uint32Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Int32Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Uint64Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                SequenceValue::Int64Sequence(s) => Some(cast_slice!(s.as_slice(), $out)),
                _ => None,
            },
            Value::Array(arr) => match arr {
                ArrayValue::FloatArray(s) => Some(cast_slice!(s, $out)),
                ArrayValue::DoubleArray(s) => Some(cast_slice!(s, $out)),
                ArrayValue::CharArray(s) => Some(cast_slice!(s, $out)),
                ArrayValue::WCharArray(s) => Some(cast_slice!(s, $out)),
                ArrayValue::OctetArray(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Uint8Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Int8Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Uint16Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Int16Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Uint32Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Int32Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Uint64Array(s) => Some(cast_slice!(s, $out)),
                ArrayValue::Int64Array(s) => Some(cast_slice!(s, $out)),
                _ => None,
            },
            Value::BoundedSequence(seq) => match seq {
                BoundedSequenceValue::FloatBoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::DoubleBoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::CharBoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::WCharBoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::OctetBoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Uint8BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Int8BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Uint16BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Int16BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Uint32BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Int32BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Uint64BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                BoundedSequenceValue::Int64BoundedSequence(s) => Some(cast_slice!(&*s, $out)),
                _ => None,
            },
            _ => None,
        }
    };
}

macro_rules! cast_slice {
    ($slice:expr, $out:ty) => {
        $slice.iter().map(|v| *v as $out).collect::<Vec<$out>>()
    };
}

macro_rules! numeric_seq_from_vec {
    ($value:expr, $values:expr) => {
        match $value {
            ValueMut::Sequence(seq) => match seq {
                SequenceValueMut::FloatSequence(s) => Some(fill_seq!(s, $values, f32)),
                SequenceValueMut::DoubleSequence(s) => Some(fill_seq!(s, $values, f64)),
                SequenceValueMut::CharSequence(s) => Some(fill_seq!(s, $values, u8)),
                SequenceValueMut::WCharSequence(s) => Some(fill_seq!(s, $values, u16)),
                SequenceValueMut::OctetSequence(s) => Some(fill_seq!(s, $values, u8)),
                SequenceValueMut::Uint8Sequence(s) => Some(fill_seq!(s, $values, u8)),
                SequenceValueMut::Int8Sequence(s) => Some(fill_seq!(s, $values, i8)),
                SequenceValueMut::Uint16Sequence(s) => Some(fill_seq!(s, $values, u16)),
                SequenceValueMut::Int16Sequence(s) => Some(fill_seq!(s, $values, i16)),
                SequenceValueMut::Uint32Sequence(s) => Some(fill_seq!(s, $values, u32)),
                SequenceValueMut::Int32Sequence(s) => Some(fill_seq!(s, $values, i32)),
                SequenceValueMut::Uint64Sequence(s) => Some(fill_seq!(s, $values, u64)),
                SequenceValueMut::Int64Sequence(s) => Some(fill_seq!(s, $values, i64)),
                _ => None,
            },
            ValueMut::Array(arr) => match arr {
                ArrayValueMut::FloatArray(s) => Some(fill_slice!(s, $values, f32)),
                ArrayValueMut::DoubleArray(s) => Some(fill_slice!(s, $values, f64)),
                ArrayValueMut::CharArray(s) => Some(fill_slice!(s, $values, u8)),
                ArrayValueMut::WCharArray(s) => Some(fill_slice!(s, $values, u16)),
                ArrayValueMut::OctetArray(s) => Some(fill_slice!(s, $values, u8)),
                ArrayValueMut::Uint8Array(s) => Some(fill_slice!(s, $values, u8)),
                ArrayValueMut::Int8Array(s) => Some(fill_slice!(s, $values, i8)),
                ArrayValueMut::Uint16Array(s) => Some(fill_slice!(s, $values, u16)),
                ArrayValueMut::Int16Array(s) => Some(fill_slice!(s, $values, i16)),
                ArrayValueMut::Uint32Array(s) => Some(fill_slice!(s, $values, u32)),
                ArrayValueMut::Int32Array(s) => Some(fill_slice!(s, $values, i32)),
                ArrayValueMut::Uint64Array(s) => Some(fill_slice!(s, $values, u64)),
                ArrayValueMut::Int64Array(s) => Some(fill_slice!(s, $values, i64)),
                _ => None,
            },
            ValueMut::BoundedSequence(seq) => match seq {
                BoundedSequenceValueMut::FloatBoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, f32))
                }
                BoundedSequenceValueMut::DoubleBoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, f64))
                }
                BoundedSequenceValueMut::CharBoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u8))
                }
                BoundedSequenceValueMut::WCharBoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u16))
                }
                BoundedSequenceValueMut::OctetBoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u8))
                }
                BoundedSequenceValueMut::Uint8BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u8))
                }
                BoundedSequenceValueMut::Int8BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, i8))
                }
                BoundedSequenceValueMut::Uint16BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u16))
                }
                BoundedSequenceValueMut::Int16BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, i16))
                }
                BoundedSequenceValueMut::Uint32BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u32))
                }
                BoundedSequenceValueMut::Int32BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, i32))
                }
                BoundedSequenceValueMut::Uint64BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, u64))
                }
                BoundedSequenceValueMut::Int64BoundedSequence(s) => {
                    Some(fill_bounded!(s, $values, i64))
                }
                _ => None,
            },
            _ => None,
        }
    };
}

macro_rules! fill_seq {
    ($seq:expr, $values:expr, $elem:ty) => {{
        let converted: Vec<$elem> = $values.iter().map(|v| *v as $elem).collect();
        *$seq = Sequence::from(&converted[..]);
    }};
}

macro_rules! fill_slice {
    ($slice:expr, $values:expr, $elem:ty) => {{
        for (slot, v) in $slice.iter_mut().zip($values) {
            *slot = *v as $elem;
        }
    }};
}

macro_rules! fill_bounded {
    ($seq:expr, $values:expr, $elem:ty) => {{
        let mut seq = $seq;
        let _ = seq.try_reset($values.len().min(seq.upper_bound()));
        for (slot, v) in seq.as_mut_slice().iter_mut().zip($values) {
            *slot = *v as $elem;
        }
    }};
}

pub fn read_f64_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<f64>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(value) => numeric_seq_to_vec!(value, f64)
            .ok_or_else(|| err(format!("expected numeric sequence field `{field}`"))),
    }
}

pub fn read_i64_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<i64>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(value) => numeric_seq_to_vec!(value, i64)
            .ok_or_else(|| err(format!("expected numeric sequence field `{field}`"))),
    }
}

pub fn write_f64_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[f64],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(slot) => numeric_seq_from_vec!(slot, values)
            .ok_or_else(|| err(format!("expected mut numeric sequence field `{field}`"))),
    }
}

pub fn write_i64_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[i64],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(slot) => numeric_seq_from_vec!(slot, values)
            .ok_or_else(|| err(format!("expected mut numeric sequence field `{field}`"))),
    }
}

pub fn read_f32_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<f32>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(value) => numeric_seq_to_vec!(value, f32)
            .ok_or_else(|| err(format!("expected numeric sequence field `{field}`"))),
    }
}

pub fn read_i32_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<i32>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(value) => numeric_seq_to_vec!(value, i32)
            .ok_or_else(|| err(format!("expected numeric sequence field `{field}`"))),
    }
}

pub fn read_u32_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<u32>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(value) => numeric_seq_to_vec!(value, u32)
            .ok_or_else(|| err(format!("expected numeric sequence field `{field}`"))),
    }
}

pub fn read_u64_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<u64>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(value) => numeric_seq_to_vec!(value, u64)
            .ok_or_else(|| err(format!("expected numeric sequence field `{field}`"))),
    }
}

pub fn write_f32_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[f32],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(slot) => numeric_seq_from_vec!(slot, values)
            .ok_or_else(|| err(format!("expected mut numeric sequence field `{field}`"))),
    }
}

pub fn write_i32_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[i32],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(slot) => numeric_seq_from_vec!(slot, values)
            .ok_or_else(|| err(format!("expected mut numeric sequence field `{field}`"))),
    }
}

pub fn write_u32_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[u32],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(slot) => numeric_seq_from_vec!(slot, values)
            .ok_or_else(|| err(format!("expected mut numeric sequence field `{field}`"))),
    }
}

pub fn write_u64_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[u64],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(slot) => numeric_seq_from_vec!(slot, values)
            .ok_or_else(|| err(format!("expected mut numeric sequence field `{field}`"))),
    }
}

// --- byte sequences (protobuf `bytes` ↔ ROS `uint8[]` / `octet[]` / `int8[]`) ---

pub fn read_byte_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<u8>> {
    match view.get(field) {
        Some(Value::Sequence(SequenceValue::Uint8Sequence(seq)))
        | Some(Value::Sequence(SequenceValue::OctetSequence(seq)))
        | Some(Value::Sequence(SequenceValue::CharSequence(seq))) => Ok(seq.as_slice().to_vec()),
        Some(Value::Sequence(SequenceValue::Int8Sequence(seq))) => {
            Ok(seq.as_slice().iter().map(|v| *v as u8).collect())
        }
        Some(Value::Array(ArrayValue::Int8Array(items))) => {
            Ok(items.iter().map(|v| *v as u8).collect())
        }
        _ => Ok(read_i64_seq(view, field)?
            .into_iter()
            .map(|v| v as u8)
            .collect()),
    }
}

pub fn write_byte_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    data: &[u8],
) -> Result<()> {
    let written = match view.get_mut(field) {
        Some(ValueMut::Sequence(SequenceValueMut::Uint8Sequence(seq)))
        | Some(ValueMut::Sequence(SequenceValueMut::OctetSequence(seq)))
        | Some(ValueMut::Sequence(SequenceValueMut::CharSequence(seq))) => {
            *seq = Sequence::from(data);
            true
        }
        Some(ValueMut::Sequence(SequenceValueMut::Int8Sequence(seq))) => {
            let signed: Vec<i8> = data.iter().map(|v| *v as i8).collect();
            *seq = Sequence::from(&signed[..]);
            true
        }
        Some(ValueMut::Array(ArrayValueMut::Int8Array(items))) => {
            for (slot, v) in items.iter_mut().zip(data) {
                *slot = *v as i8;
            }
            true
        }
        _ => false,
    };
    if written {
        return Ok(());
    }
    let widened: Vec<i64> = data.iter().map(|v| i64::from(*v)).collect();
    write_i64_seq(view, field, &widened)
}

// --- string sequences ---

pub fn read_string_seq(view: &DynamicMessageView<'_>, field: &str) -> Result<Vec<String>> {
    match view.get(field) {
        None => Ok(Vec::new()),
        Some(Value::Sequence(SequenceValue::StringSequence(seq))) => {
            Ok(seq.as_slice().iter().map(|s| s.to_string()).collect())
        }
        Some(Value::Sequence(SequenceValue::WStringSequence(seq))) => {
            Ok(seq.as_slice().iter().map(|s| s.to_string()).collect())
        }
        Some(Value::Sequence(SequenceValue::BoundedStringSequence(seq))) => {
            Ok(seq.iter().map(|s| s.to_string()).collect())
        }
        Some(Value::Array(ArrayValue::StringArray(items))) => {
            Ok(items.iter().map(|s| s.to_string()).collect())
        }
        Some(Value::BoundedSequence(BoundedSequenceValue::StringBoundedSequence(seq))) => {
            Ok(seq.iter().map(|s| s.to_string()).collect())
        }
        Some(other) => Err(err(format!(
            "expected string sequence field `{field}`, got {other:?}"
        ))),
    }
}

pub fn write_string_seq(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    values: &[String],
) -> Result<()> {
    match view.get_mut(field) {
        None => Ok(()),
        Some(ValueMut::Sequence(SequenceValueMut::StringSequence(seq))) => {
            let mut fresh: Sequence<rosidl_runtime_rs::String> = Sequence::new(values.len());
            for (slot, v) in fresh.as_mut_slice().iter_mut().zip(values) {
                *slot = v.as_str().into();
            }
            *seq = fresh;
            Ok(())
        }
        Some(ValueMut::Sequence(SequenceValueMut::WStringSequence(seq))) => {
            let mut fresh: Sequence<rosidl_runtime_rs::WString> = Sequence::new(values.len());
            for (slot, v) in fresh.as_mut_slice().iter_mut().zip(values) {
                *slot = v.as_str().into();
            }
            *seq = fresh;
            Ok(())
        }
        Some(ValueMut::Sequence(SequenceValueMut::BoundedStringSequence(mut seq))) => {
            seq.reset(values.len());
            for (slot, v) in seq.as_mut_slice().iter_mut().zip(values) {
                let _ = slot.try_assign(v.as_str());
            }
            Ok(())
        }
        Some(ValueMut::Array(ArrayValueMut::StringArray(items))) => {
            for (slot, v) in items.iter_mut().zip(values) {
                *slot = v.as_str().into();
            }
            Ok(())
        }
        Some(ValueMut::BoundedSequence(BoundedSequenceValueMut::StringBoundedSequence(
            mut seq,
        ))) => {
            let _ = seq.try_reset(values.len().min(seq.upper_bound()));
            for (slot, v) in seq.as_mut_slice().iter_mut().zip(values) {
                *slot = v.as_str().into();
            }
            Ok(())
        }
        Some(other) => Err(err(format!(
            "expected mut string sequence field `{field}`, got {other:?}"
        ))),
    }
}

// --- google.protobuf.Timestamp / Duration ↔ builtin_interfaces Time / Duration ---

pub fn read_timestamp(
    view: &DynamicMessageView<'_>,
    field: &str,
) -> Result<Option<Timestamp>> {
    let Some(stamp) = nested_view(view, field)? else {
        return Ok(None);
    };
    Ok(Some(Timestamp {
        seconds: read_i64(&stamp, "sec")?,
        nanos: read_i32(&stamp, "nanosec")?,
    }))
}

pub fn write_timestamp(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    ts: &Timestamp,
) -> Result<()> {
    with_nested_mut(view, field, |stamp| {
        write_i64(stamp, "sec", ts.seconds)?;
        write_i64(stamp, "nanosec", i64::from(ts.nanos))
    })
}

pub fn read_duration(
    view: &DynamicMessageView<'_>,
    field: &str,
) -> Result<Option<ProstDuration>> {
    let Some(d) = nested_view(view, field)? else {
        return Ok(None);
    };
    Ok(Some(ProstDuration {
        seconds: read_i64(&d, "sec")?,
        nanos: read_i32(&d, "nanosec")?,
    }))
}

pub fn write_duration(
    view: &mut DynamicMessageViewMut<'_>,
    field: &str,
    d: &ProstDuration,
) -> Result<()> {
    with_nested_mut(view, field, |slot| {
        write_i64(slot, "sec", d.seconds)?;
        write_i64(slot, "nanosec", i64::from(d.nanos))
    })
}
