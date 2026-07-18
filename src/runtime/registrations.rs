//! Registration types for sockets managed by [`super::Executor`].

use std::sync::Arc;
use std::time::{Duration, Instant};

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::runtime::callback_group::CallbackGroup;
use crate::runtime::queues::ActionMessageCallback;
use crate::zmq_helpers::{
    apply_action_options_with, apply_rpc_options_with, HighWaterMark,
};

pub type MessageCallback = Arc<dyn Fn(&str, &[u8]) + Send + Sync>;
pub type ServiceHandler = Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>;
pub type ActionGoalHandler =
    Arc<dyn Fn(&[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync>;

pub enum RegistrationKind {
    Sub,
    Service,
    Action,
    ActionClient,
}

pub struct SubRegistration {
    pub socket: Socket,
    pub endpoint: String,
}

impl SubRegistration {
    pub fn kind(&self) -> RegistrationKind {
        RegistrationKind::Sub
    }
}

pub struct ServiceRegistration {
    pub socket: Socket,
    pub service_name: String,
    pub handler: ServiceHandler,
    pub callback_group: CallbackGroup,
    pub endpoint: String,
    pub identity: Vec<u8>,
    pub heartbeat_interval: Duration,
    pub last_heartbeat: Instant,
}

impl ServiceRegistration {
    pub fn create(
        context: &Context,
        service_name: &str,
        handler: ServiceHandler,
        callback_group: CallbackGroup,
        endpoint: &str,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
        hwm: HighWaterMark,
    ) -> crate::errors::Result<Self> {
        let socket = context.socket(SocketType::DEALER)?;
        apply_rpc_options_with(&socket, hwm)?;
        let worker_id = identity
            .map(str::to_string)
            .unwrap_or_else(|| format!("worker-{}", &Uuid::new_v4().simple().to_string()[..8]));
        let identity = worker_id.into_bytes();
        socket.set_identity(&identity)?;
        socket.connect(endpoint)?;
        let mut reg = Self {
            socket,
            service_name: service_name.to_string(),
            handler,
            callback_group,
            endpoint: endpoint.to_string(),
            identity,
            heartbeat_interval: Duration::from_millis(heartbeat_interval_ms),
            last_heartbeat: Instant::now(),
        };
        reg.send_control(b"READY")?;
        reg.last_heartbeat = Instant::now();
        Ok(reg)
    }

    pub fn kind(&self) -> RegistrationKind {
        RegistrationKind::Service
    }

    pub fn send_control(&self, command: &[u8]) -> crate::errors::Result<()> {
        self.socket
            .send_multipart([command, self.service_name.as_bytes()], 0)?;
        Ok(())
    }

    pub fn send_heartbeat(&self) -> crate::errors::Result<()> {
        self.send_control(b"HEARTBEAT")
    }

    pub fn disconnect(&self) {
        let _ = self.send_control(b"DISCONNECT");
    }
}

pub struct ActionRegistration {
    pub socket: Socket,
    pub action_name: String,
    pub handler: ActionGoalHandler,
    pub callback_group: CallbackGroup,
    pub endpoint: String,
    pub identity: Vec<u8>,
    pub heartbeat_interval: Duration,
    pub last_heartbeat: Instant,
}

impl ActionRegistration {
    pub fn create(
        context: &Context,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: CallbackGroup,
        endpoint: &str,
        identity: Option<&str>,
        heartbeat_interval_ms: u64,
        hwm: HighWaterMark,
    ) -> crate::errors::Result<Self> {
        let socket = context.socket(SocketType::DEALER)?;
        apply_action_options_with(&socket, hwm)?;
        let worker_id = identity
            .map(str::to_string)
            .unwrap_or_else(|| format!("worker-{}", &Uuid::new_v4().simple().to_string()[..8]));
        let identity = worker_id.into_bytes();
        socket.set_identity(&identity)?;
        socket.connect(endpoint)?;
        let mut reg = Self {
            socket,
            action_name: action_name.to_string(),
            handler,
            callback_group,
            endpoint: endpoint.to_string(),
            identity,
            heartbeat_interval: Duration::from_millis(heartbeat_interval_ms),
            last_heartbeat: Instant::now(),
        };
        reg.send_control(b"READY")?;
        reg.last_heartbeat = Instant::now();
        Ok(reg)
    }

    pub fn kind(&self) -> RegistrationKind {
        RegistrationKind::Action
    }

    pub fn send_control(&self, command: &[u8]) -> crate::errors::Result<()> {
        self.socket
            .send_multipart([command, self.action_name.as_bytes()], 0)?;
        Ok(())
    }

    pub fn send_heartbeat(&self) -> crate::errors::Result<()> {
        self.send_control(b"HEARTBEAT")
    }

    pub fn disconnect(&self) {
        let _ = self.send_control(b"DISCONNECT");
    }
}

pub struct ActionClientRegistration {
    pub socket: Socket,
    pub endpoint: String,
    pub goal_callbacks: std::collections::HashMap<String, ActionMessageCallback>,
}

impl ActionClientRegistration {
    pub fn create(
        context: &Context,
        endpoint: &str,
        hwm: HighWaterMark,
    ) -> crate::errors::Result<Self> {
        let socket = context.socket(SocketType::DEALER)?;
        apply_action_options_with(&socket, hwm)?;
        socket.connect(endpoint)?;
        Ok(Self {
            socket,
            endpoint: endpoint.to_string(),
            goal_callbacks: std::collections::HashMap::new(),
        })
    }

    pub fn kind(&self) -> RegistrationKind {
        RegistrationKind::ActionClient
    }
}

pub enum Registration {
    Sub(SubRegistration),
    Service(ServiceRegistration),
    Action(ActionRegistration),
    ActionClient(ActionClientRegistration),
}

impl Registration {
    pub fn socket(&self) -> &Socket {
        match self {
            Self::Sub(reg) => &reg.socket,
            Self::Service(reg) => &reg.socket,
            Self::Action(reg) => &reg.socket,
            Self::ActionClient(reg) => &reg.socket,
        }
    }

    pub fn kind(&self) -> RegistrationKind {
        match self {
            Self::Sub(reg) => reg.kind(),
            Self::Service(reg) => reg.kind(),
            Self::Action(reg) => reg.kind(),
            Self::ActionClient(reg) => reg.kind(),
        }
    }
}
