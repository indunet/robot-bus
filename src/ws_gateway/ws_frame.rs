//! Binary framing for multiplexed WebSocket RPC (V3: one connection, many streams).
//!
//! Frame layout (little-endian):
//! - REQUEST: `u8 type` | `u32 stream_id` | `u8 opcode` | opcode-specific header | body
//!   - Subscribe (1): `u16 topic_len` | topic UTF-8 | `i32 qos_depth`
//!   - Publish (2): `u16 topic_len` | topic UTF-8 | body = raw bus payload
//!   - Call (3): `u16 name_len` | name | `u32 timeout_ms` | `u16 id_len` | request_id | body
//!   - SendGoal (4): `u16 name_len` | name | `u16 goal_id_len` | goal_id | `u32 timeout_ms` | body
//! - DATA: `u8 type` | `u32 stream_id` | `u32 payload_len` | payload
//!   - Subscribe payload: `u16 topic_len` | topic | raw bus payload
//!   - Call payload: raw response bytes
//!   - SendGoal payload: `u8 kind` | raw body
//! - CANCEL: `u8 type` | `u32 stream_id` | `u32 payload_len=0`
//! - TRAILER: `u8 type` | `u32 stream_id` | `u32 payload_len` | `u32 status` | UTF-8 message
//!
//! Clients allocate odd `stream_id` values (HTTP/2 style). TRAILER ends a stream.
//! Publish success is TRAILER only (no DATA ack). Status codes match historical
//! gRPC / tonic codes (0 = OK).

pub const FRAME_REQUEST: u8 = 1;
pub const FRAME_DATA: u8 = 2;
pub const FRAME_CANCEL: u8 = 3;
pub const FRAME_TRAILER: u8 = 4;
pub const FRAME_PING: u8 = 5;
pub const FRAME_PONG: u8 = 6;

pub const OPCODE_SUBSCRIBE: u8 = 1;
pub const OPCODE_PUBLISH: u8 = 2;
pub const OPCODE_CALL: u8 = 3;
pub const OPCODE_SEND_GOAL: u8 = 4;

pub const ACTION_KIND_GOAL: u8 = 1;
pub const ACTION_KIND_FEEDBACK: u8 = 2;
pub const ACTION_KIND_RESULT: u8 = 3;
pub const ACTION_KIND_CANCEL: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Subscribe = OPCODE_SUBSCRIBE,
    Publish = OPCODE_PUBLISH,
    Call = OPCODE_CALL,
    SendGoal = OPCODE_SEND_GOAL,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Result<Self, FrameError> {
        match v {
            OPCODE_SUBSCRIBE => Ok(Self::Subscribe),
            OPCODE_PUBLISH => Ok(Self::Publish),
            OPCODE_CALL => Ok(Self::Call),
            OPCODE_SEND_GOAL => Ok(Self::SendGoal),
            other => Err(FrameError::UnknownOpcode(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestHeader {
    Subscribe {
        topic: String,
        qos_depth: i32,
    },
    Publish {
        topic: String,
    },
    Call {
        service_name: String,
        timeout_ms: u32,
        request_id: String,
    },
    SendGoal {
        action_name: String,
        goal_id: String,
        timeout_ms: u32,
    },
}

impl RequestHeader {
    pub fn opcode(&self) -> Opcode {
        match self {
            Self::Subscribe { .. } => Opcode::Subscribe,
            Self::Publish { .. } => Opcode::Publish,
            Self::Call { .. } => Opcode::Call,
            Self::SendGoal { .. } => Opcode::SendGoal,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Frame {
    Request {
        stream_id: u32,
        header: RequestHeader,
        body: Vec<u8>,
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
    #[error("unknown opcode {0}")]
    UnknownOpcode(u8),
    #[error("invalid utf-8 in frame")]
    InvalidUtf8,
    #[error("string too long")]
    StringTooLong,
}

pub fn encode_frame(frame: &Frame) -> Result<Vec<u8>, FrameError> {
    match frame {
        Frame::Request {
            stream_id,
            header,
            body,
        } => encode_request(*stream_id, header, body),
        Frame::Data { stream_id, payload } => encode_data(*stream_id, payload),
        Frame::Cancel { stream_id } => encode_empty(FRAME_CANCEL, *stream_id),
        Frame::Trailer {
            stream_id,
            status,
            message,
        } => {
            let inner_len = 4 + message.len();
            let mut out = Vec::with_capacity(1 + 4 + 4 + inner_len);
            out.push(FRAME_TRAILER);
            out.extend_from_slice(&stream_id.to_le_bytes());
            out.extend_from_slice(&(inner_len as u32).to_le_bytes());
            out.extend_from_slice(&status.to_le_bytes());
            out.extend_from_slice(message.as_bytes());
            Ok(out)
        }
        Frame::Ping { stream_id } => encode_empty(FRAME_PING, *stream_id),
        Frame::Pong { stream_id } => encode_empty(FRAME_PONG, *stream_id),
    }
}

/// One-allocation Subscribe DATA frame: topic header + raw bus payload.
pub fn encode_subscribe_data(
    stream_id: u32,
    topic: &str,
    payload: &[u8],
) -> Result<Vec<u8>, FrameError> {
    if topic.len() > u16::MAX as usize {
        return Err(FrameError::StringTooLong);
    }
    let inner_len = 2 + topic.len() + payload.len();
    let mut out = Vec::with_capacity(1 + 4 + 4 + inner_len);
    out.push(FRAME_DATA);
    out.extend_from_slice(&stream_id.to_le_bytes());
    out.extend_from_slice(&(inner_len as u32).to_le_bytes());
    out.extend_from_slice(&(topic.len() as u16).to_le_bytes());
    out.extend_from_slice(topic.as_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

/// One-allocation SendGoal DATA frame: kind byte + raw body.
pub fn encode_action_data(stream_id: u32, kind: u8, body: &[u8]) -> Vec<u8> {
    let inner_len = 1 + body.len();
    let mut out = Vec::with_capacity(1 + 4 + 4 + inner_len);
    out.push(FRAME_DATA);
    out.extend_from_slice(&stream_id.to_le_bytes());
    out.extend_from_slice(&(inner_len as u32).to_le_bytes());
    out.push(kind);
    out.extend_from_slice(body);
    out
}

pub fn decode_subscribe_data(payload: &[u8]) -> Result<(String, Vec<u8>), FrameError> {
    if payload.len() < 2 {
        return Err(FrameError::Truncated);
    }
    let topic_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + topic_len {
        return Err(FrameError::Truncated);
    }
    let topic = std::str::from_utf8(&payload[2..2 + topic_len])
        .map_err(|_| FrameError::InvalidUtf8)?
        .to_string();
    Ok((topic, payload[2 + topic_len..].to_vec()))
}

pub fn decode_action_data(payload: &[u8]) -> Result<(u8, Vec<u8>), FrameError> {
    if payload.is_empty() {
        return Err(FrameError::Truncated);
    }
    Ok((payload[0], payload[1..].to_vec()))
}

fn encode_request(
    stream_id: u32,
    header: &RequestHeader,
    body: &[u8],
) -> Result<Vec<u8>, FrameError> {
    let mut out = Vec::new();
    out.push(FRAME_REQUEST);
    out.extend_from_slice(&stream_id.to_le_bytes());
    out.push(header.opcode() as u8);
    match header {
        RequestHeader::Subscribe { topic, qos_depth } => {
            push_str(&mut out, topic)?;
            out.extend_from_slice(&qos_depth.to_le_bytes());
        }
        RequestHeader::Publish { topic } => {
            push_str(&mut out, topic)?;
            out.extend_from_slice(body);
        }
        RequestHeader::Call {
            service_name,
            timeout_ms,
            request_id,
        } => {
            push_str(&mut out, service_name)?;
            out.extend_from_slice(&timeout_ms.to_le_bytes());
            push_str(&mut out, request_id)?;
            out.extend_from_slice(body);
        }
        RequestHeader::SendGoal {
            action_name,
            goal_id,
            timeout_ms,
        } => {
            push_str(&mut out, action_name)?;
            push_str(&mut out, goal_id)?;
            out.extend_from_slice(&timeout_ms.to_le_bytes());
            out.extend_from_slice(body);
        }
    }
    Ok(out)
}

fn encode_data(stream_id: u32, payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    let mut out = Vec::with_capacity(1 + 4 + 4 + payload.len());
    out.push(FRAME_DATA);
    out.extend_from_slice(&stream_id.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn encode_empty(ty: u8, stream_id: u32) -> Result<Vec<u8>, FrameError> {
    let mut out = Vec::with_capacity(1 + 4 + 4);
    out.push(ty);
    out.extend_from_slice(&stream_id.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    Ok(out)
}

fn push_str(out: &mut Vec<u8>, s: &str) -> Result<(), FrameError> {
    if s.len() > u16::MAX as usize {
        return Err(FrameError::StringTooLong);
    }
    out.extend_from_slice(&(s.len() as u16).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

pub fn decode_frame(bytes: &[u8]) -> Result<Frame, FrameError> {
    if bytes.len() < 1 + 4 {
        return Err(FrameError::Truncated);
    }
    let ty = bytes[0];
    let stream_id = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
    match ty {
        FRAME_REQUEST => decode_request(stream_id, &bytes[5..]),
        FRAME_DATA => {
            let payload = read_len_prefixed(&bytes[5..])?;
            Ok(Frame::Data { stream_id, payload })
        }
        FRAME_CANCEL => Ok(Frame::Cancel { stream_id }),
        FRAME_PING => Ok(Frame::Ping { stream_id }),
        FRAME_PONG => Ok(Frame::Pong { stream_id }),
        FRAME_TRAILER => {
            let payload = read_len_prefixed(&bytes[5..])?;
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

fn decode_request(stream_id: u32, rest: &[u8]) -> Result<Frame, FrameError> {
    if rest.is_empty() {
        return Err(FrameError::Truncated);
    }
    let opcode = Opcode::from_u8(rest[0])?;
    let mut cur = &rest[1..];
    let header_body = match opcode {
        Opcode::Subscribe => {
            let topic = read_str(&mut cur)?;
            let qos_depth = read_i32(&mut cur)?;
            (RequestHeader::Subscribe { topic, qos_depth }, Vec::new())
        }
        Opcode::Publish => {
            let topic = read_str(&mut cur)?;
            (RequestHeader::Publish { topic }, cur.to_vec())
        }
        Opcode::Call => {
            let service_name = read_str(&mut cur)?;
            let timeout_ms = read_u32(&mut cur)?;
            let request_id = read_str(&mut cur)?;
            (
                RequestHeader::Call {
                    service_name,
                    timeout_ms,
                    request_id,
                },
                cur.to_vec(),
            )
        }
        Opcode::SendGoal => {
            let action_name = read_str(&mut cur)?;
            let goal_id = read_str(&mut cur)?;
            let timeout_ms = read_u32(&mut cur)?;
            (
                RequestHeader::SendGoal {
                    action_name,
                    goal_id,
                    timeout_ms,
                },
                cur.to_vec(),
            )
        }
    };
    Ok(Frame::Request {
        stream_id,
        header: header_body.0,
        body: header_body.1,
    })
}

fn read_str(cur: &mut &[u8]) -> Result<String, FrameError> {
    if cur.len() < 2 {
        return Err(FrameError::Truncated);
    }
    let len = u16::from_le_bytes([cur[0], cur[1]]) as usize;
    if cur.len() < 2 + len {
        return Err(FrameError::Truncated);
    }
    let s = std::str::from_utf8(&cur[2..2 + len]).map_err(|_| FrameError::InvalidUtf8)?;
    *cur = &cur[2 + len..];
    Ok(s.to_string())
}

fn read_u32(cur: &mut &[u8]) -> Result<u32, FrameError> {
    if cur.len() < 4 {
        return Err(FrameError::Truncated);
    }
    let v = u32::from_le_bytes([cur[0], cur[1], cur[2], cur[3]]);
    *cur = &cur[4..];
    Ok(v)
}

fn read_i32(cur: &mut &[u8]) -> Result<i32, FrameError> {
    if cur.len() < 4 {
        return Err(FrameError::Truncated);
    }
    let v = i32::from_le_bytes([cur[0], cur[1], cur[2], cur[3]]);
    *cur = &cur[4..];
    Ok(v)
}

fn read_len_prefixed(bytes: &[u8]) -> Result<Vec<u8>, FrameError> {
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
    fn roundtrip_all_opcodes() {
        let cases = [
            Frame::Request {
                stream_id: 1,
                header: RequestHeader::Publish {
                    topic: "/cmd".into(),
                },
                body: vec![1, 2, 3],
            },
            Frame::Request {
                stream_id: 3,
                header: RequestHeader::Subscribe {
                    topic: "/imu".into(),
                    qos_depth: 8,
                },
                body: vec![],
            },
            Frame::Request {
                stream_id: 5,
                header: RequestHeader::Call {
                    service_name: "svc.echo".into(),
                    timeout_ms: 1000,
                    request_id: "r1".into(),
                },
                body: b"ping".to_vec(),
            },
            Frame::Request {
                stream_id: 7,
                header: RequestHeader::SendGoal {
                    action_name: "act.nav".into(),
                    goal_id: "g1".into(),
                    timeout_ms: 5000,
                },
                body: b"go".to_vec(),
            },
        ];
        for frame in cases {
            let bytes = encode_frame(&frame).unwrap();
            match (frame, decode_frame(&bytes).unwrap()) {
                (
                    Frame::Request {
                        stream_id: a,
                        header: ha,
                        body: ba,
                    },
                    Frame::Request {
                        stream_id: b,
                        header: hb,
                        body: bb,
                    },
                ) => {
                    assert_eq!(a, b);
                    assert_eq!(ha, hb);
                    assert_eq!(ba, bb);
                }
                _ => panic!("opcode roundtrip mismatch"),
            }
        }
    }

    #[test]
    fn roundtrip_subscribe_data_topic_header() {
        let bytes = encode_subscribe_data(3, "ws.sub", b"hello-ws").unwrap();
        match decode_frame(&bytes).unwrap() {
            Frame::Data { stream_id, payload } => {
                assert_eq!(stream_id, 3);
                let (topic, body) = decode_subscribe_data(&payload).unwrap();
                assert_eq!(topic, "ws.sub");
                assert_eq!(body, b"hello-ws");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn roundtrip_action_data_kind() {
        let bytes = encode_action_data(9, ACTION_KIND_RESULT, b"done");
        match decode_frame(&bytes).unwrap() {
            Frame::Data { stream_id, payload } => {
                assert_eq!(stream_id, 9);
                let (kind, body) = decode_action_data(&payload).unwrap();
                assert_eq!(kind, ACTION_KIND_RESULT);
                assert_eq!(body, b"done");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn roundtrip_data_cancel_trailer() {
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

        match decode_frame(&encode_frame(&Frame::Cancel { stream_id: 5 }).unwrap()).unwrap() {
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
        match decode_frame(&encode_frame(&Frame::Ping { stream_id: 0 }).unwrap()).unwrap() {
            Frame::Ping { stream_id } => assert_eq!(stream_id, 0),
            other => panic!("unexpected {other:?}"),
        }
        match decode_frame(&encode_frame(&Frame::Pong { stream_id: 0 }).unwrap()).unwrap() {
            Frame::Pong { stream_id } => assert_eq!(stream_id, 0),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn unknown_opcode_errors() {
        let mut bytes = vec![FRAME_REQUEST, 1, 0, 0, 0, 99];
        bytes.extend_from_slice(&[0, 0]);
        match decode_frame(&bytes) {
            Err(FrameError::UnknownOpcode(99)) => {}
            other => panic!("expected UnknownOpcode, got {other:?}"),
        }
    }

    #[test]
    fn truncated_request_errors() {
        assert!(matches!(
            decode_frame(&[FRAME_REQUEST, 1, 0, 0, 0]),
            Err(FrameError::Truncated)
        ));
    }

    #[test]
    fn multiplex_stream_ids_independent() {
        let a = encode_subscribe_data(1, "imu", b"a").unwrap();
        let b = encode_subscribe_data(3, "odom", b"b").unwrap();
        assert_ne!(
            decode_frame(&a).unwrap().stream_id(),
            decode_frame(&b).unwrap().stream_id()
        );
    }
}
