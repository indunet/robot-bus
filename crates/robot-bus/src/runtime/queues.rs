//! Thread-safe queues between user threads, worker pool, and the I/O thread.

use std::sync::Arc;

use crate::action_bus::ActionMessage;

pub enum OutboundCommand {
    SendGoal {
        action_name: String,
        goal_id: String,
        body: Vec<u8>,
        callback: ActionMessageCallback,
    },
    CancelGoal {
        action_name: String,
        goal_id: String,
        body: Vec<u8>,
    },
}

pub struct ServiceReply {
    pub client_id: Vec<u8>,
    pub service: Vec<u8>,
    pub request_id: Vec<u8>,
    pub body: Vec<u8>,
}

pub struct ActionReply {
    pub client_id: Vec<u8>,
    pub goal_id: Vec<u8>,
    pub kind: Vec<u8>,
    pub body: Vec<u8>,
}

pub enum ReplyMessage {
    Service {
        service_name: String,
        reply: ServiceReply,
    },
    Action {
        action_name: String,
        reply: ActionReply,
    },
}

pub type ActionMessageCallback = Arc<dyn Fn(&ActionMessage) + Send + Sync>;
