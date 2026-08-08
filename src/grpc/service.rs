//! `ServiceGateway` — unary Call bridged to a ZMQ service-bus REQ client.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use tonic::{Request, Response, Status};
use zmq::Context;

use crate::errors::BusError;
use crate::service_bus::ServiceClient;
use crate::zmq_helpers::HighWaterMark;

use super::pb::service_gateway_server::ServiceGateway;
use super::pb::{ServiceCallRequest, ServiceCallResponse};

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

    fn checkout(&self) -> Result<ServiceClient, Status> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| Status::internal("service client pool mutex poisoned"))?;
        while guard.is_empty() {
            guard = self
                .available
                .wait(guard)
                .map_err(|_| Status::internal("service client pool condvar poisoned"))?;
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

    pub async fn call_service(&self, req: ServiceCallRequest) -> Result<Vec<u8>, Status> {
        if req.service_name.is_empty() {
            return Err(Status::invalid_argument("service_name is required"));
        }

        let pool = Arc::clone(&self.pool);
        let service_name = req.service_name;
        let body = req.request;
        let request_id = if req.request_id.is_empty() {
            None
        } else {
            Some(req.request_id)
        };
        let timeout = timeout_from_ms(req.timeout_ms);

        tokio::task::spawn_blocking(move || {
            let client = pool.checkout()?;
            let result = client
                .call(&service_name, &body, request_id.as_deref(), timeout)
                .map_err(bus_status);
            pool.checkin(client);
            result
        })
        .await
        .map_err(|err| Status::internal(format!("service call join: {err}")))?
    }
}

fn bus_status(err: BusError) -> Status {
    match err {
        BusError::Timeout(msg) => Status::deadline_exceeded(msg),
        BusError::NoWorker { name } => Status::unavailable(format!("no worker for '{name}'")),
        BusError::WorkerDied { name } => Status::unavailable(format!("worker died for '{name}'")),
        BusError::Cancelled { name } => Status::cancelled(format!("cancelled '{name}'")),
        other => Status::internal(other.to_string()),
    }
}

fn timeout_from_ms(timeout_ms: u32) -> Option<Duration> {
    if timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(u64::from(timeout_ms)))
    }
}

#[tonic::async_trait]
impl ServiceGateway for ServiceGatewayService {
    async fn call(
        &self,
        request: Request<ServiceCallRequest>,
    ) -> Result<Response<ServiceCallResponse>, Status> {
        let response = self.call_service(request.into_inner()).await?;
        Ok(Response::new(ServiceCallResponse { response }))
    }
}
