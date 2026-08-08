//! Binary framing for browser gRPC-over-WebSocket (V1: one RPC per connection).
//!
//! Frame layout (little-endian):
//! - REQUEST: `u8 type` | `u16 method_len` | method UTF-8 | `u32 payload_len` | payload
//! - DATA / CANCEL / TRAILER: `u8 type` | `u32 payload_len` | payload
//!
//! TRAILER payload: `u32 status` (0 = OK, tonic/grpc status codes) | UTF-8 message.

pub const FRAME_REQUEST: u8 = 1;
pub const FRAME_DATA: u8 = 2;
pub const FRAME_CANCEL: u8 = 3;
pub const FRAME_TRAILER: u8 = 4;

pub const METHOD_SUBSCRIBE: &str = "robot_bus_interface.grpc.v1.MessageGateway/Subscribe";
pub const METHOD_PUBLISH: &str = "robot_bus_interface.grpc.v1.MessageGateway/Publish";
pub const METHOD_CALL: &str = "robot_bus_interface.grpc.v1.ServiceGateway/Call";
pub const METHOD_SEND_GOAL: &str = "robot_bus_interface.grpc.v1.ActionGateway/SendGoal";

#[derive(Debug, Clone)]
pub enum Frame {
    Request { method: String, payload: Vec<u8> },
    Data { payload: Vec<u8> },
    Cancel,
    Trailer { status: u32, message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("truncated websocket frame")]
    Truncated,
    #[error("unknown frame type {0}")]
    UnknownType(u8),
    #[error("invalid utf-8 in frame")]
    InvalidUtf8,
    #[error("method too long")]
    MethodTooLong,
}

pub fn encode_frame(frame: &Frame) -> Result<Vec<u8>, FrameError> {
    match frame {
        Frame::Request { method, payload } => {
            if method.len() > u16::MAX as usize {
                return Err(FrameError::MethodTooLong);
            }
            let mut out = Vec::with_capacity(1 + 2 + method.len() + 4 + payload.len());
            out.push(FRAME_REQUEST);
            out.extend_from_slice(&(method.len() as u16).to_le_bytes());
            out.extend_from_slice(method.as_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(payload);
            Ok(out)
        }
        Frame::Data { payload } => {
            let mut out = Vec::with_capacity(1 + 4 + payload.len());
            out.push(FRAME_DATA);
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(payload);
            Ok(out)
        }
        Frame::Cancel => Ok(vec![FRAME_CANCEL, 0, 0, 0, 0]),
        Frame::Trailer { status, message } => {
            let mut payload = Vec::with_capacity(4 + message.len());
            payload.extend_from_slice(&status.to_le_bytes());
            payload.extend_from_slice(message.as_bytes());
            let mut out = Vec::with_capacity(1 + 4 + payload.len());
            out.push(FRAME_TRAILER);
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&payload);
            Ok(out)
        }
    }
}

pub fn decode_frame(bytes: &[u8]) -> Result<Frame, FrameError> {
    if bytes.is_empty() {
        return Err(FrameError::Truncated);
    }
    let ty = bytes[0];
    match ty {
        FRAME_REQUEST => {
            if bytes.len() < 1 + 2 {
                return Err(FrameError::Truncated);
            }
            let method_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
            let method_start = 3;
            let method_end = method_start + method_len;
            if bytes.len() < method_end + 4 {
                return Err(FrameError::Truncated);
            }
            let method = std::str::from_utf8(&bytes[method_start..method_end])
                .map_err(|_| FrameError::InvalidUtf8)?
                .to_string();
            let payload_len = u32::from_le_bytes([
                bytes[method_end],
                bytes[method_end + 1],
                bytes[method_end + 2],
                bytes[method_end + 3],
            ]) as usize;
            let payload_start = method_end + 4;
            let payload_end = payload_start + payload_len;
            if bytes.len() < payload_end {
                return Err(FrameError::Truncated);
            }
            Ok(Frame::Request {
                method,
                payload: bytes[payload_start..payload_end].to_vec(),
            })
        }
        FRAME_DATA => {
            let (payload, _) = read_payload(&bytes[1..])?;
            Ok(Frame::Data { payload })
        }
        FRAME_CANCEL => Ok(Frame::Cancel),
        FRAME_TRAILER => {
            let (payload, _) = read_payload(&bytes[1..])?;
            if payload.len() < 4 {
                return Err(FrameError::Truncated);
            }
            let status = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
            let message = std::str::from_utf8(&payload[4..])
                .map_err(|_| FrameError::InvalidUtf8)?
                .to_string();
            Ok(Frame::Trailer { status, message })
        }
        other => Err(FrameError::UnknownType(other)),
    }
}

fn read_payload(bytes: &[u8]) -> Result<(Vec<u8>, usize), FrameError> {
    if bytes.len() < 4 {
        return Err(FrameError::Truncated);
    }
    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if bytes.len() < 4 + len {
        return Err(FrameError::Truncated);
    }
    Ok((bytes[4..4 + len].to_vec(), 4 + len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_request_and_trailer() {
        let req = Frame::Request {
            method: METHOD_PUBLISH.to_string(),
            payload: vec![1, 2, 3],
        };
        let bytes = encode_frame(&req).unwrap();
        match decode_frame(&bytes).unwrap() {
            Frame::Request { method, payload } => {
                assert_eq!(method, METHOD_PUBLISH);
                assert_eq!(payload, vec![1, 2, 3]);
            }
            other => panic!("unexpected {other:?}"),
        }

        let tr = Frame::Trailer {
            status: 0,
            message: "ok".into(),
        };
        let bytes = encode_frame(&tr).unwrap();
        match decode_frame(&bytes).unwrap() {
            Frame::Trailer { status, message } => {
                assert_eq!(status, 0);
                assert_eq!(message, "ok");
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
