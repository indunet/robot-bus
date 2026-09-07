//! `ServiceGateway` — unary Call bridged to a ZMQ service-bus REQ client.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use zmq::Context;

use crate::errors::BusError;
use crate::service_bus::ServiceClient;
use crate::zmq_helpers::HighWaterMark;

use super::rpc_status::{Code, RpcStatus};

const SERVICE_CLIENT_POOL_SIZE: usize = 8;
const MAX_PENDING_CALLS: usize = 256;

struct ServiceClientPool {
    frontend: String,
    state: Mutex<VecDeque<ServiceClient>>,
    available: Arc<Semaphore>,
    admission: Arc<Semaphore>,
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
            available: Arc::new(Semaphore::new(SERVICE_CLIENT_POOL_SIZE)),
            admission: Arc::new(Semaphore::new(SERVICE_CLIENT_POOL_SIZE + MAX_PENDING_CALLS)),
        })
    }

    fn checkout(self: &Arc<Self>, permit: OwnedSemaphorePermit) -> Result<ClientLease, RpcStatus> {
        let client = self
            .state
            .lock()
            .map_err(|_| RpcStatus::internal("service client pool mutex poisoned"))?
            .pop_front()
            .ok_or_else(|| RpcStatus::internal("service client pool permit mismatch"))?;
        Ok(ClientLease {
            pool: Arc::clone(self),
            client: Some(client),
            _permit: permit,
        })
    }

    fn checkin(&self, client: ServiceClient) {
        let Ok(mut guard) = self.state.lock() else {
            return;
        };
        guard.push_back(client);
    }
}

// Return the client before releasing its permit, including during unwinding.
struct ClientLease {
    pool: Arc<ServiceClientPool>,
    client: Option<ServiceClient>,
    _permit: OwnedSemaphorePermit,
}

impl Drop for ClientLease {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.checkin(client);
        }
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
        let deadline = timeout_from_ms(timeout_ms).map(|timeout| Instant::now() + timeout);
        // Bound both waiting calls and calls still executing after caller timeout.
        let admission = Arc::clone(&pool.admission)
            .try_acquire_owned()
            .map_err(|_| {
                RpcStatus::new(Code::ResourceExhausted, format!("busy '{service_name}'"))
            })?;
        let call = async move {
            let permit = Arc::clone(&pool.available)
                .acquire_owned()
                .await
                .map_err(|_| RpcStatus::unavailable("service client pool closed"))?;
            tokio::task::spawn_blocking(move || {
                let _admission = admission;
                let lease = pool.checkout(permit)?;
                // spawn_blocking may itself have queued: never dispatch an expired request.
                remaining_time(deadline)?;
                lease
                    .client
                    .as_ref()
                    .expect("leased client")
                    .call_with_deadline(&service_name, &body, request_id.as_deref(), deadline)
                    .map_err(bus_status)
            })
            .await
            .map_err(|err| RpcStatus::internal(format!("service call join: {err}")))?
        };
        match deadline {
            Some(deadline) => tokio::time::timeout_at(deadline.into(), call)
                .await
                .map_err(|_| RpcStatus::deadline_exceeded("service call deadline exceeded"))?,
            None => call.await,
        }
    }
}

fn bus_status(err: BusError) -> RpcStatus {
    match err {
        BusError::Busy { name } => {
            RpcStatus::new(Code::ResourceExhausted, format!("busy '{name}'"))
        }
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

fn remaining_time(deadline: Option<Instant>) -> Result<Option<Duration>, RpcStatus> {
    deadline
        .map(|deadline| {
            deadline
                .checked_duration_since(Instant::now())
                .filter(|remaining| !remaining.is_zero())
                .ok_or_else(|| {
                    RpcStatus::deadline_exceeded("service call deadline exceeded before dispatch")
                })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gateway() -> (ServiceGatewayService, zmq::Socket) {
        let context = Context::new();
        let server = context.socket(zmq::REP).unwrap();
        server.set_linger(0).unwrap();
        server.bind("tcp://127.0.0.1:*").unwrap();
        let endpoint = server.get_last_endpoint().unwrap().unwrap();
        (ServiceGatewayService::new(endpoint), server)
    }

    #[tokio::test]
    async fn deadline_includes_waiting_for_a_pool_client() {
        let (gateway, server) = gateway();
        let held = Arc::clone(&gateway.pool.available)
            .acquire_many_owned(SERVICE_CLIENT_POOL_SIZE as u32)
            .await
            .unwrap();
        let start = Instant::now();
        let result = gateway
            .call_service("echo".into(), vec![], "expired".into(), 40)
            .await;
        drop(held);
        assert_eq!(result.unwrap_err().code(), Code::DeadlineExceeded);
        assert!(start.elapsed() < Duration::from_millis(500));
        assert_eq!(
            gateway.pool.admission.available_permits(),
            SERVICE_CLIENT_POOL_SIZE + MAX_PENDING_CALLS
        );
        assert!(
            !crate::zmq_helpers::poll_readable(&server, 100).unwrap(),
            "expired call was dispatched"
        );
    }

    #[tokio::test]
    async fn full_admission_queue_fails_immediately() {
        let (gateway, _server) = gateway();
        let _held = Arc::clone(&gateway.pool.admission)
            .acquire_many_owned((SERVICE_CLIENT_POOL_SIZE + MAX_PENDING_CALLS) as u32)
            .await
            .unwrap();
        let result = gateway
            .call_service("echo".into(), vec![], String::new(), 0)
            .await;
        assert_eq!(result.unwrap_err().code(), Code::ResourceExhausted);
    }

    #[test]
    fn expired_call_in_blocking_queue_is_never_dispatched() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(1)
            .build()
            .unwrap();
        let (gateway, server) = gateway();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let blocker = runtime.spawn_blocking(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let result =
            runtime.block_on(gateway.call_service("echo".into(), vec![], String::new(), 40));
        release_tx.send(()).unwrap();
        runtime.block_on(blocker).unwrap();
        runtime.shutdown_timeout(Duration::from_secs(2));
        assert_eq!(result.unwrap_err().code(), Code::DeadlineExceeded);
        assert_eq!(
            gateway.pool.available.available_permits(),
            SERVICE_CLIENT_POOL_SIZE
        );
        assert_eq!(
            gateway.pool.admission.available_permits(),
            SERVICE_CLIENT_POOL_SIZE + MAX_PENDING_CALLS
        );
        assert!(
            !crate::zmq_helpers::poll_readable(&server, 100).unwrap(),
            "expired blocking job was dispatched"
        );
    }
}
