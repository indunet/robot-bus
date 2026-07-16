//! `ServiceGateway` — unary Call bridged to a ZMQ service-bus REQ client.

use std::time::Duration;

use tonic::{Request, Response, Status};

use crate::errors::BusError;
use crate::service_bus::ServiceClient;

use super::pb::service_gateway_server::ServiceGateway;
use super::pb::{ServiceCallRequest, ServiceCallResponse};

#[derive(Clone, Debug)]
pub struct ServiceGatewayService {
    service_frontend: String,
}

impl ServiceGatewayService {
    pub fn new(service_frontend: impl Into<String>) -> Self {
        Self {
            service_frontend: service_frontend.into(),
        }
    }
}

fn bus_status(err: BusError) -> Status {
    match err {
        BusError::Timeout(msg) => Status::deadline_exceeded(msg),
        BusError::NoWorker { name } => Status::unavailable(format!("no worker for '{name}'")),
        BusError::WorkerDied { name } => Status::unavailable(format!("worker died for '{name}'")),
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
        let req = request.into_inner();
        if req.service_name.is_empty() {
            return Err(Status::invalid_argument("service_name is required"));
        }

        let frontend = self.service_frontend.clone();
        let service_name = req.service_name;
        let body = req.request;
        let request_id = if req.request_id.is_empty() {
            None
        } else {
            Some(req.request_id)
        };
        let timeout = timeout_from_ms(req.timeout_ms);

        let response = tokio::task::spawn_blocking(move || {
            let client = ServiceClient::new(Some(&frontend)).map_err(bus_status)?;
            let response = client
                .call(
                    &service_name,
                    &body,
                    request_id.as_deref(),
                    timeout,
                )
                .map_err(bus_status)?;
            Ok::<_, Status>(response)
        })
        .await
        .map_err(|err| Status::internal(format!("service call join: {err}")))??;

        Ok(Response::new(ServiceCallResponse { response }))
    }
}
