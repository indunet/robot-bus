//! `ServiceGateway` — unary Call bridged to a ZMQ service-bus REQ client.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use zmq::Context;

use crate::errors::BusError;
use crate::service_bus::ServiceClient;
use crate::zmq_helpers::HighWaterMark;

use super::rpc_status::RpcStatus;

const SERVICE_CLIENT_POOL_SIZE: usize = 8;

struct ServiceClientPool {
    frontend: String,
    state: Mutex<VecDeque<ServiceClient>>,
    available: Condvar,
}

impl ServiceClientPool {
    fn new(frontend: String) -> Result<Self, BusError> {
        let context = Context::new();
        let mut idle = VecDeque::with_capacity(SERVICE_CLIENT_POOL_SIZE);
        for _ in 0..SERVICE_CLIENT_POOL_SIZE {
            idle.push_back(ServiceClient::with_context_hwm(
                &context,
                Some(&frontend),
                HighWaterMark::RPC,
            )?);
        }
        Ok(Self {
            frontend,
            state: Mutex::new(idle),
            available: Condvar::new(),
        })
    }

    fn checkout(&self) -> Result<ServiceClient, RpcStatus> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| RpcStatus::internal("service client pool mutex poisoned"))?;
        while guard.is_empty() {
            guard = self
                .available
                .wait(guard)
                .map_err(|_| RpcStatus::internal("service client pool condvar poisoned"))?;
        }
        Ok(guard.pop_front().expect("non-empty after wait"))
    }

    fn checkin(&self, client: ServiceClient) {
        let Ok(mut guard) = self.state.lock() else {
            return;
        };
        guard.push_back(client);
        self.available.notify_one();
    }
}

#[derive(Clone)]
pub struct ServiceGatewayService {
    pool: Arc<ServiceClientPool>,
}

impl std::fmt::Debug for ServiceGatewayService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceGatewayService")
            .field("frontend", &self.pool.frontend)
            .field("pool_size", &SERVICE_CLIENT_POOL_SIZE)
            .finish()
    }
}

impl ServiceGatewayService {
    pub fn new(service_frontend: impl Into<String>) -> Self {
        let frontend = service_frontend.into();
        let pool = ServiceClientPool::new(frontend)
            .unwrap_or_else(|err| panic!("service client pool init failed: {err}"));
        Self {
            pool: Arc::new(pool),
        }
    }

    pub async fn call_service(
        &self,
        service_name: String,
        body: Vec<u8>,
        request_id: String,
        timeout_ms: u32,
    ) -> Result<Vec<u8>, RpcStatus> {
        if service_name.is_empty() {
            return Err(RpcStatus::invalid_argument("service_name is required"));
        }

        let pool = Arc::clone(&self.pool);
        let request_id = if request_id.is_empty() {
            None
        } else {
            Some(request_id)
        };
        let timeout = timeout_from_ms(timeout_ms);

        tokio::task::spawn_blocking(move || {
            let client = pool.checkout()?;
            let result = client
                .call(&service_name, &body, request_id.as_deref(), timeout)
                .map_err(bus_status);
            pool.checkin(client);
            result
        })
        .await
        .map_err(|err| RpcStatus::internal(format!("service call join: {err}")))?
    }
}

fn bus_status(err: BusError) -> RpcStatus {
    match err {
        BusError::Timeout(msg) => RpcStatus::deadline_exceeded(msg),
        BusError::NoWorker { name } => RpcStatus::unavailable(format!("no worker for '{name}'")),
        BusError::WorkerDied { name } => {
            RpcStatus::unavailable(format!("worker died for '{name}'"))
        }
        BusError::Cancelled { name } => RpcStatus::cancelled(format!("cancelled '{name}'")),
        other => RpcStatus::internal(other.to_string()),
    }
}

fn timeout_from_ms(timeout_ms: u32) -> Option<Duration> {
    if timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(u64::from(timeout_ms)))
    }
}
