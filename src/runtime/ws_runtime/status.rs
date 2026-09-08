//! Map WebSocket RPC trailer status codes onto [`BusError`].

use crate::action_bus::ActionKind;
use crate::errors::{BusError, Result};
use crate::ws::rpc_status::Code;
use crate::ws::ws_frame::{
    ACTION_KIND_CANCEL, ACTION_KIND_FEEDBACK, ACTION_KIND_GOAL, ACTION_KIND_RESULT,
};

pub(super) fn ws_action_kind(kind: u8) -> Result<ActionKind> {
    match kind {
        ACTION_KIND_GOAL => Ok(ActionKind::Goal),
        ACTION_KIND_FEEDBACK => Ok(ActionKind::Feedback),
        ACTION_KIND_RESULT => Ok(ActionKind::Result),
        ACTION_KIND_CANCEL => Ok(ActionKind::Cancel),
        other => Err(BusError::Protocol(format!(
            "unknown ws action kind: {other}"
        ))),
    }
}

fn cancelled_name(message: &str) -> String {
    message
        .strip_prefix("cancelled ")
        .unwrap_or(message)
        .trim_matches('\'')
        .to_string()
}

pub(super) fn map_rpc_status(status: u32, message: &str) -> BusError {
    match Code::from_u32(status) {
        Code::ResourceExhausted => BusError::Busy {
            name: message
                .strip_prefix("busy ")
                .unwrap_or(message)
                .trim_matches('\'')
                .to_string(),
        },
        Code::DeadlineExceeded => BusError::Timeout(message.to_string()),
        Code::Cancelled => BusError::Cancelled {
            name: cancelled_name(message),
        },
        Code::Aborted if message.starts_with("action aborted: ") => {
            BusError::ActionAborted(message["action aborted: ".len()..].to_string())
        }
        Code::FailedPrecondition if message.starts_with("action rejected: ") => {
            BusError::ActionRejected(message["action rejected: ".len()..].to_string())
        }
        Code::Internal => map_internal_rpc_status(status, message),
        Code::NotFound => {
            if let Some(rest) = message.strip_prefix("no goal ") {
                BusError::NoGoal {
                    goal_id: rest.trim_matches('\'').to_string(),
                }
            } else {
                BusError::Protocol(format!("rpc {status}: {message}"))
            }
        }
        Code::Unavailable => {
            if let Some(rest) = message.strip_prefix("no worker for ") {
                BusError::NoWorker {
                    name: rest.trim_matches('\'').to_string(),
                }
            } else if let Some(rest) = message.strip_prefix("worker died for ") {
                BusError::WorkerDied {
                    name: rest.trim_matches('\'').to_string(),
                }
            } else {
                BusError::Protocol(format!("rpc {status}: {message}"))
            }
        }
        _ => BusError::Protocol(format!("rpc {status}: {message}")),
    }
}

fn map_internal_rpc_status(status: u32, message: &str) -> BusError {
    if message.starts_with("handler panicked for ") {
        BusError::HandlerPanicked {
            name: message
                .trim_start_matches("handler panicked for ")
                .trim_matches('\'')
                .to_string(),
        }
    } else if message.starts_with("cancelled ") || message.starts_with("cancelled '") {
        BusError::Cancelled {
            name: cancelled_name(message),
        }
    } else if let Some(rest) = message.strip_prefix("action aborted: ") {
        BusError::ActionAborted(rest.to_string())
    } else if let Some(rest) = message.strip_prefix("action rejected: ") {
        BusError::ActionRejected(rest.to_string())
    } else {
        BusError::Protocol(format!("rpc {status}: {message}"))
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn callback_errors_remain_typed_across_websocket() {
        assert!(
            matches!(map_rpc_status(1, "cancelled 'motion'"), BusError::Cancelled { name } if name == "motion")
        );
        assert!(
            matches!(map_rpc_status(13, "cancelled 'motion'"), BusError::Cancelled { name } if name == "motion")
        );
        assert!(matches!(
            map_rpc_status(9, "action rejected: invalid"),
            BusError::ActionRejected(_)
        ));
        assert!(matches!(
            map_rpc_status(10, "action aborted: stopped"),
            BusError::ActionAborted(_)
        ));
        assert!(matches!(
            map_rpc_status(13, "action aborted: stopped"),
            BusError::ActionAborted(_)
        ));
        assert!(matches!(
            map_rpc_status(13, "action rejected: invalid"),
            BusError::ActionRejected(_)
        ));

        assert!(
            matches!(map_rpc_status(8, "busy 'echo'"), BusError::Busy { name } if name == "echo")
        );
        assert!(
            matches!(map_rpc_status(13, "handler panicked for 'echo'"), BusError::HandlerPanicked { name } if name == "echo")
        );
    }
}
