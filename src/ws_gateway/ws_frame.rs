//! Binary framing for multiplexed WebSocket RPC (V2: one connection, many streams).
//!
//! Frame layout (little-endian):
//! - REQUEST: `u8 type` | `u32 stream_id` | `u16 method_len` | method UTF-8 | `u32 payload_len` | payload
//! - DATA: `u8 type` | `u32 stream_id` | `u32 payload_len` | payload
//! - CANCEL: `u8 type` | `u32 stream_id` | `u32 payload_len=0`
//! - TRAILER: `u8 type` | `u32 stream_id` | `u32 payload_len` | `u32 status` | UTF-8 message
//!
//! Clients allocate odd `stream_id` values (HTTP/2 style). TRAILER ends a stream.
//! Status codes match historical gRPC / tonic codes (0 = OK).

pub const FRAME_REQUEST: u8 = 1;
pub const FRAME_DATA: u8 = 2;
pub const FRAME_CANCEL: u8 = 3;
pub const FRAME_TRAILER: u8 = 4;
pub const FRAME_PING: u8 = 5;
pub const FRAME_PONG: u8 = 6;

pub const METHOD_SUBSCRIBE: &str = "robot_bus_interfaces.grpc.v1.MessageGateway/Subscribe";
pub const METHOD_PUBLISH: &str = "robot_bus_interfaces.grpc.v1.MessageGateway/Publish";
pub const METHOD_CALL: &str = "robot_bus_interfaces.grpc.v1.ServiceGateway/Call";
pub const METHOD_SEND_GOAL: &str = "robot_bus_interfaces.grpc.v1.ActionGateway/SendGoal";

#[derive(Debug, Clone)]
pub enum Frame {
    Request {
        stream_id: u32,
        method: String,
        payload: Vec<u8>,
    },
    Data {
        stream_id: u32,
        payload: Vec<u8>,
    },
    Cancel {
        stream_id: u32,
    },
    Trailer {
        stream_id: u32,
        status: u32,
        message: String,
    },
    Ping {
        stream_id: u32,
    },
    Pong {
        stream_id: u32,
    },
}

impl Frame {
    pub fn stream_id(&self) -> u32 {
        match self {
            Frame::Request { stream_id, .. }
            | Frame::Data { stream_id, .. }
            | Frame::Cancel { stream_id }
            | Frame::Trailer { stream_id, .. }
            | Frame::Ping { stream_id }
            | Frame::Pong { stream_id } => *stream_id,
        }
    }
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
        Frame::Request {
            stream_id,
            method,
            payload,
        } => {
            if method.len() > u16::MAX as usize {
                return Err(FrameError::MethodTooLong);
            }
            let mut out = Vec::with_capacity(1 + 4 + 2 + method.len() + 4 + payload.len());
            out.push(FRAME_REQUEST);
            out.extend_from_slice(&stream_id.to_le_bytes());
            out.extend_from_slice(&(method.len() as u16).to_le_bytes());
            out.extend_from_slice(method.as_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(payload);
            Ok(out)
        }
        Frame::Data { stream_id, payload } => {
            let mut out = Vec::with_capacity(1 + 4 + 4 + payload.len());
            out.push(FRAME_DATA);
            out.extend_from_slice(&stream_id.to_le_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(payload);
            Ok(out)
        }
        Frame::Cancel { stream_id } => {
            let mut out = Vec::with_capacity(1 + 4 + 4);
            out.push(FRAME_CANCEL);
            out.extend_from_slice(&stream_id.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            Ok(out)
        }
        Frame::Trailer {
            stream_id,
            status,
            message,
        } => {
            let mut payload = Vec::with_capacity(4 + message.len());
            payload.extend_from_slice(&status.to_le_bytes());
            payload.extend_from_slice(message.as_bytes());
            let mut out = Vec::with_capacity(1 + 4 + 4 + payload.len());
            out.push(FRAME_TRAILER);
            out.extend_from_slice(&stream_id.to_le_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&payload);
            Ok(out)
        }
        Frame::Ping { stream_id } => encode_empty(FRAME_PING, *stream_id),
        Frame::Pong { stream_id } => encode_empty(FRAME_PONG, *stream_id),
    }
}

fn encode_empty(ty: u8, stream_id: u32) -> Result<Vec<u8>, FrameError> {
    let mut out = Vec::with_capacity(1 + 4 + 4);
    out.push(ty);
    out.extend_from_slice(&stream_id.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    Ok(out)
}

pub fn decode_frame(bytes: &[u8]) -> Result<Frame, FrameError> {
    if bytes.len() < 1 + 4 {
        return Err(FrameError::Truncated);
    }
    let ty = bytes[0];
    let stream_id = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
    match ty {
        FRAME_REQUEST => {
            if bytes.len() < 1 + 4 + 2 {
                return Err(FrameError::Truncated);
            }
            let method_len = u16::from_le_bytes([bytes[5], bytes[6]]) as usize;
            let method_start = 7;
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
                stream_id,
                method,
                payload: bytes[payload_start..payload_end].to_vec(),
            })
        }
        FRAME_DATA => {
            let payload = read_payload_at(&bytes[5..])?;
            Ok(Frame::Data { stream_id, payload })
        }
        FRAME_CANCEL => Ok(Frame::Cancel { stream_id }),
        FRAME_PING => Ok(Frame::Ping { stream_id }),
        FRAME_PONG => Ok(Frame::Pong { stream_id }),
        FRAME_TRAILER => {
            let payload = read_payload_at(&bytes[5..])?;
            if payload.len() < 4 {
                return Err(FrameError::Truncated);
            }
            let status = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
            let message = std::str::from_utf8(&payload[4..])
                .map_err(|_| FrameError::InvalidUtf8)?
                .to_string();
            Ok(Frame::Trailer {
                stream_id,
                status,
                message,
            })
        }
        other => Err(FrameError::UnknownType(other)),
    }
}

fn read_payload_at(bytes: &[u8]) -> Result<Vec<u8>, FrameError> {
    if bytes.len() < 4 {
        return Err(FrameError::Truncated);
    }
    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if bytes.len() < 4 + len {
        return Err(FrameError::Truncated);
    }
    Ok(bytes[4..4 + len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_request_data_cancel_trailer() {
        let req = Frame::Request {
            stream_id: 1,
            method: METHOD_PUBLISH.to_string(),
            payload: vec![1, 2, 3],
        };
        let bytes = encode_frame(&req).unwrap();
        match decode_frame(&bytes).unwrap() {
            Frame::Request {
                stream_id,
                method,
                payload,
            } => {
                assert_eq!(stream_id, 1);
                assert_eq!(method, METHOD_PUBLISH);
                assert_eq!(payload, vec![1, 2, 3]);
            }
            other => panic!("unexpected {other:?}"),
        }

        let data = Frame::Data {
            stream_id: 3,
            payload: vec![9],
        };
        match decode_frame(&encode_frame(&data).unwrap()).unwrap() {
            Frame::Data { stream_id, payload } => {
                assert_eq!(stream_id, 3);
                assert_eq!(payload, vec![9]);
            }
            other => panic!("unexpected {other:?}"),
        }

        let cancel = Frame::Cancel { stream_id: 5 };
        match decode_frame(&encode_frame(&cancel).unwrap()).unwrap() {
            Frame::Cancel { stream_id } => assert_eq!(stream_id, 5),
            other => panic!("unexpected {other:?}"),
        }

        let tr = Frame::Trailer {
            stream_id: 7,
            status: 0,
            message: "ok".into(),
        };
        match decode_frame(&encode_frame(&tr).unwrap()).unwrap() {
            Frame::Trailer {
                stream_id,
                status,
                message,
            } => {
                assert_eq!(stream_id, 7);
                assert_eq!(status, 0);
                assert_eq!(message, "ok");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn roundtrip_ping_pong() {
        let ping = Frame::Ping { stream_id: 0 };
        match decode_frame(&encode_frame(&ping).unwrap()).unwrap() {
            Frame::Ping { stream_id } => assert_eq!(stream_id, 0),
            other => panic!("unexpected {other:?}"),
        }
        let pong = Frame::Pong { stream_id: 0 };
        match decode_frame(&encode_frame(&pong).unwrap()).unwrap() {
            Frame::Pong { stream_id } => assert_eq!(stream_id, 0),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn multiplex_stream_ids_independent() {
        let a = encode_frame(&Frame::Data {
            stream_id: 1,
            payload: b"imu".to_vec(),
        })
        .unwrap();
        let b = encode_frame(&Frame::Data {
            stream_id: 3,
            payload: b"odom".to_vec(),
        })
        .unwrap();
        assert_ne!(
            decode_frame(&a).unwrap().stream_id(),
            decode_frame(&b).unwrap().stream_id()
        );
    }
}
