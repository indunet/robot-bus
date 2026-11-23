use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use prost::Message;

use crate::errors::Result;
use crate::runtime::callback_group::CallbackGroup;
use crate::runtime::qos::QosProfile;
use crate::runtime::queues::ActionMessageCallback;
use crate::runtime::registrations::{ActionGoalHandler, ActionGoalLiveHandler};
use crate::typed::{Action, ActionOutcome};
use crate::zmq_helpers::HighWaterMark;

use super::super::Node;
use super::super::action_clients::{
    ActionClientInner, NodeActionClient, NodeActionClientRaw, NodeActionServer,
};
use super::super::options::ws_mode_unsupported;

impl Node {
    /// Register a typed action server (ROS 2–style `create_action_server`).
    pub fn create_action_server<A, F>(
        &mut self,
        action_name: &str,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer>
    where
        A: Action,
        F: Fn(A::Goal) -> ActionOutcome<A> + Send + Sync + 'static,
    {
        let cb: ActionGoalHandler = Arc::new(move |body| match A::Goal::decode(body) {
            Ok(goal) => {
                let outcome = handler(goal);
                let mut replies = Vec::with_capacity(outcome.feedbacks.len() + 1);
                for fb in outcome.feedbacks {
                    replies.push(("FEEDBACK".into(), fb.encode_to_vec()));
                }
                replies.push(("RESULT".into(), outcome.result.encode_to_vec()));
                replies
            }
            Err(err) => {
                log::warn!("typed action goal decode failed: {err}");
                vec![("RESULT".into(), Vec::new())]
            }
        });
        self.create_action_server_raw(action_name, cb, callback_group)
    }

    /// Register a typed action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_with_qos<A, F>(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: F,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer>
    where
        A: Action,
        F: Fn(A::Goal) -> ActionOutcome<A> + Send + Sync + 'static,
    {
        let cb: ActionGoalHandler = Arc::new(move |body| match A::Goal::decode(body) {
            Ok(goal) => {
                let outcome = handler(goal);
                let mut replies = Vec::with_capacity(outcome.feedbacks.len() + 1);
                for fb in outcome.feedbacks {
                    replies.push(("FEEDBACK".into(), fb.encode_to_vec()));
                }
                replies.push(("RESULT".into(), outcome.result.encode_to_vec()));
                replies
            }
            Err(err) => {
                log::warn!("typed action goal decode failed: {err}");
                vec![("RESULT".into(), Vec::new())]
            }
        });
        self.create_action_server_raw_with_qos(action_name, qos, cb, callback_group)
    }

    /// Register a raw-bytes action server.
    pub fn create_action_server_raw(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_inner(action_name, handler, callback_group, None)
    }

    /// Register a raw-bytes action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_raw_with_qos(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_inner(
            action_name,
            handler,
            callback_group,
            Some(qos.to_hwm()),
        )
    }

    fn create_action_server_raw_inner(
        &mut self,
        action_name: &str,
        handler: ActionGoalHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeActionServer> {
        self.finish_action_server(
            action_name,
            callback_group,
            hwm,
            |exec, group, endpoint, hwm| {
                exec.register_action(action_name, handler, group, Some(endpoint), None, hwm)
            },
        )
    }

    /// Register a live action server: handler may publish FEEDBACK and poll CANCEL.
    pub fn create_action_server_raw_live(
        &mut self,
        action_name: &str,
        handler: ActionGoalLiveHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_live_with_qos_inner(
            action_name,
            handler,
            callback_group,
            None,
        )
    }

    /// Live action server with KeepLast depth → DEALER HWM.
    pub fn create_action_server_raw_live_with_qos(
        &mut self,
        action_name: &str,
        qos: QosProfile,
        handler: ActionGoalLiveHandler,
        callback_group: Option<&CallbackGroup>,
    ) -> Result<NodeActionServer> {
        self.create_action_server_raw_live_with_qos_inner(
            action_name,
            handler,
            callback_group,
            Some(qos.to_hwm()),
        )
    }

    fn create_action_server_raw_live_with_qos_inner(
        &mut self,
        action_name: &str,
        handler: ActionGoalLiveHandler,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
    ) -> Result<NodeActionServer> {
        self.finish_action_server(
            action_name,
            callback_group,
            hwm,
            |exec, group, endpoint, hwm| {
                exec.register_action_live(action_name, handler, group, Some(endpoint), None, hwm)
            },
        )
    }

    fn finish_action_server(
        &mut self,
        action_name: &str,
        callback_group: Option<&CallbackGroup>,
        hwm: Option<HighWaterMark>,
        register: impl FnOnce(
            &mut crate::runtime::Executor,
            CallbackGroup,
            &str,
            Option<HighWaterMark>,
        ) -> Result<u64>,
    ) -> Result<NodeActionServer> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("create_action_server"));
        }
        self.ensure_connected()?;
        let endpoint = self.options.action_backend_endpoint()?;
        let group = callback_group
            .cloned()
            .unwrap_or_else(|| self.default_callback_group.clone());
        let id = register(&mut *self.lock_executor()?, group, &endpoint, hwm)?;
        let topology = self.start_topology_guard("action_server", action_name);
        self.topology_actions.insert(id, topology);
        Ok(NodeActionServer {
            id,
            action_name: action_name.to_string(),
        })
    }

    /// Destroy an action server created by [`create_action_server`](Self::create_action_server).
    /// Same `start()` constraint as [`cancel_timer`](Self::cancel_timer).
    pub fn destroy_action_server(&mut self, handle: &NodeActionServer) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("destroy_action_server"));
        }
        self.topology_actions.remove(&handle.id);
        self.lock_executor()?.destroy_action_server(handle.id)
    }

    /// Create a typed action client (ROS 2–style `create_action_client`).
    pub fn create_action_client<A: Action>(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClient<A>> {
        Ok(NodeActionClient {
            inner: self.create_action_client_raw(action_name)?,
            _marker: PhantomData,
        })
    }

    /// Create a typed action client with KeepLast depth → DEALER HWM.
    pub fn create_action_client_with_qos<A: Action>(
        &mut self,
        action_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeActionClient<A>> {
        Ok(NodeActionClient {
            inner: self.create_action_client_raw_with_qos(action_name, qos)?,
            _marker: PhantomData,
        })
    }

    /// Create a raw-bytes action client bound to `action_name`.
    pub fn create_action_client_raw(
        &mut self,
        action_name: impl Into<String>,
    ) -> Result<NodeActionClientRaw> {
        self.create_action_client_raw_with_hwm(action_name, self.client_action_hwm())
    }

    /// Create a raw-bytes action client with KeepLast depth → DEALER HWM.
    pub fn create_action_client_raw_with_qos(
        &mut self,
        action_name: impl Into<String>,
        qos: QosProfile,
    ) -> Result<NodeActionClientRaw> {
        self.create_action_client_raw_with_hwm(action_name, qos.to_hwm())
    }

    /// Like [`create_action_client_raw`](Self::create_action_client_raw), with an explicit HWM.
    pub fn create_action_client_raw_with_hwm(
        &mut self,
        action_name: impl Into<String>,
        hwm: HighWaterMark,
    ) -> Result<NodeActionClientRaw> {
        let action_name = action_name.into();
        self.ensure_connected()?;
        let topology = Some(self.start_topology_guard("action_client", &action_name));
        #[cfg(feature = "ws")]
        if self.options.is_ws() {
            let ctx = self.ensure_ws()?.client_context();
            return Ok(NodeActionClientRaw {
                inner: ActionClientInner::Ws(ctx),
                action_name,
                console_url: self.console_url_opt(),
                _topology: topology,
            });
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        Ok(NodeActionClientRaw {
            inner: ActionClientInner::Zmq {
                context: self.context.clone_zmq(),
                endpoint,
                hwm: Mutex::new(hwm),
            },
            action_name,
            console_url: self.console_url_opt(),
            _topology: topology,
        })
    }

    fn client_action_hwm(&self) -> HighWaterMark {
        match &self.executor {
            Some(exec) => exec
                .lock()
                .map(|e| e.action_hwm())
                .unwrap_or(HighWaterMark::ACTION),
            None => HighWaterMark::ACTION,
        }
    }

    /// Connect the executor-owned action client used by callback-style [`send_goal`](Self::send_goal).
    pub fn connect_action_client(&mut self) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported(
                "connect_action_client (use create_action_client)",
            ));
        }
        let endpoint = self.options.action_frontend_endpoint()?;
        self.lock_executor()?.connect_action_client(Some(&endpoint))
    }

    /// Submit a goal via the executor (callback receives FEEDBACK / RESULT). Prefer
    /// [`create_action_client`](Self::create_action_client) for a ROS 2–style sync handle.
    pub fn send_goal(
        &mut self,
        action_name: &str,
        body: &[u8],
        callback: ActionMessageCallback,
        goal_id: Option<&str>,
    ) -> Result<String> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported("send_goal (use create_action_client)"));
        }
        self.lock_executor()?
            .send_goal(action_name, body, callback, goal_id)
    }

    pub fn cancel_goal(&mut self, action_name: &str, goal_id: &str, body: &[u8]) -> Result<()> {
        if self.options.is_ws() {
            return Err(ws_mode_unsupported(
                "cancel_goal (use create_action_client)",
            ));
        }
        self.lock_executor()?
            .cancel_goal(action_name, goal_id, body)
    }
}
