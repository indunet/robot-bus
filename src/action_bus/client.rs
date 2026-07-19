//! DEALER client for the action bus frontend.

use std::time::Duration;

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::{parse_error_body, BusError, Result};
use crate::transports;
use crate::zmq_helpers::{apply_action_options_with, poll_readable, HighWaterMark};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Goal,
    Feedback,
    Result,
    Cancel,
}

impl ActionKind {
    pub(crate) fn from_wire(s: &str) -> Result<Self> {
        match s {
            "GOAL" => Ok(Self::Goal),
            "FEEDBACK" => Ok(Self::Feedback),
            "RESULT" => Ok(Self::Result),
            "CANCEL" => Ok(Self::Cancel),
            other => Err(BusError::Protocol(format!("unknown action kind: {other:?}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMessage {
    pub action_name: String,
    pub goal_id: String,
    pub kind: ActionKind,
    pub body: Vec<u8>,
}

pub struct ActionClient {
    endpoint: String,
    socket: Socket,
}

impl ActionClient {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::ACTION)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        Self::with_context_hwm(&Context::new(), endpoint, hwm)
    }

    /// Create a client using a shared ZeroMQ context (required for inproc).
    pub fn with_context_hwm(
        context: &Context,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::action_frontend_endpoint("localhost", "tcp")
                .map_err(|e| BusError::Protocol(e))?,
        };
        let socket = context.socket(SocketType::DEALER)?;
        apply_action_options_with(&socket, hwm)?;
        socket.connect(&endpoint)?;
        log::info!("action client connected to {endpoint}");
        Ok(Self { endpoint, socket })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        Ok(HighWaterMark::from_socket(&self.socket)?)
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        hwm.apply(&self.socket)?;
        Ok(())
    }

    pub fn send_goal(
        &self,
        action_name: &str,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ActionMessage>> {
        self.iter_goal(action_name, body, goal_id, timeout)
            .collect()
    }

    pub fn iter_goal(
        &self,
        action_name: &str,
        body: &[u8],
        goal_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> ActionGoalIter<'_> {
        let gid = self
            .submit_goal(action_name, body, goal_id)
            .unwrap_or_else(|_| {
                goal_id
                    .map(str::to_string)
                    .unwrap_or_else(|| Uuid::new_v4().simple().to_string())
            });
        ActionGoalIter {
            client: self,
            action_name: action_name.to_string(),
            goal_id: gid,
            timeout,
            done: false,
        }
    }

    /// Send a GOAL frame without waiting for replies. Returns the goal id used.
    pub fn submit_goal(
        &self,
        action_name: &str,
        body: &[u8],
        goal_id: Option<&str>,
    ) -> Result<String> {
        let gid = goal_id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        self.socket.send_multipart(
            [
                action_name.as_bytes(),
                gid.as_bytes(),
                b"GOAL",
                body,
            ],
            0,
        )?;
        Ok(gid)
    }

    /// Send a CANCEL frame without waiting for RESULT.
    pub fn submit_cancel(
        &self,
        action_name: &str,
        goal_id: &str,
        body: &[u8],
    ) -> Result<()> {
        self.socket.send_multipart(
            [
                action_name.as_bytes(),
                goal_id.as_bytes(),
                b"CANCEL",
                body,
            ],
            0,
        )?;
        Ok(())
    }

    pub fn cancel(
        &self,
        action_name: &str,
        goal_id: &str,
        body: &[u8],
        timeout: Option<Duration>,
    ) -> Result<ActionMessage> {
        self.submit_cancel(action_name, goal_id, body)?;
        loop {
            let msg = self.recv_message(timeout)?;
            if msg.action_name != action_name || msg.goal_id != goal_id {
                continue;
            }
            if msg.kind != ActionKind::Result {
                continue;
            }
            if let Some(err) = parse_error_body(&msg.body) {
                return Err(err);
            }
            return Ok(msg);
        }
    }

    /// Receive one action-bus reply frame (optionally with a poll timeout).
    pub fn recv_message(&self, timeout: Option<Duration>) -> Result<ActionMessage> {
        if let Some(duration) = timeout {
            let ms = duration.as_millis().min(i64::MAX as u128) as i64;
            if !poll_readable(&self.socket, ms)? {
                return Err(BusError::Timeout(format!(
                    "action client timed out after {}s",
                    duration.as_secs_f64()
                )));
            }
        }
        let frames = self.socket.recv_multipart(0)?;
        if frames.len() != 4 {
            return Err(BusError::Protocol(format!(
                "expected 4 reply frames, got {}",
                frames.len()
            )));
        }
        Ok(ActionMessage {
            action_name: String::from_utf8_lossy(&frames[0]).into_owned(),
            goal_id: String::from_utf8_lossy(&frames[1]).into_owned(),
            kind: ActionKind::from_wire(&String::from_utf8_lossy(&frames[2]))?,
            body: frames[3].clone(),
        })
    }
}

pub struct ActionGoalIter<'a> {
    client: &'a ActionClient,
    action_name: String,
    goal_id: String,
    timeout: Option<Duration>,
    done: bool,
}

impl Iterator for ActionGoalIter<'_> {
    type Item = Result<ActionMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let msg = match self.client.recv_message(self.timeout) {
            Ok(msg) => msg,
            Err(err) => return Some(Err(err)),
        };
        if msg.action_name != self.action_name || msg.goal_id != self.goal_id {
            return Some(Err(BusError::Protocol(format!(
                "unexpected message for {:?}/{:?}",
                msg.action_name, msg.goal_id
            ))));
        }
        if msg.kind == ActionKind::Result {
            if let Some(err) = parse_error_body(&msg.body) {
                return Some(Err(err));
            }
            self.done = true;
        }
        Some(Ok(msg))
    }
}

impl Drop for ActionClient {
    fn drop(&mut self) {
        let _ = self.socket.set_linger(0);
    }
}
