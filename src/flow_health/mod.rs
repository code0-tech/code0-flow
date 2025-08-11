use futures_core::Stream;
use std::pin::Pin;

use tonic::{Request, Response, Status};
use tonic_health::pb::{
    HealthCheckRequest, HealthCheckResponse, health_check_response::ServingStatus,
    health_server::Health,
};

#[derive(Debug)]
pub struct HealthService {
    nats_url: String,
}

impl HealthService {
    pub fn new(nats_url: String) -> Self {
        Self { nats_url }
    }

    async fn check_nats_connection(&self) -> bool {
        match async_nats::connect(&self.nats_url).await {
            Ok(_client) => {
                // Successfully connected to NATS
                true
            }
            Err(_) => false,
        }
    }
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let service = request.into_inner().service.to_lowercase();

        match service.as_str() {
            "liveness" => Ok(Response::new(HealthCheckResponse {
                status: ServingStatus::Serving as i32,
            })),
            "readiness" => {
                let nats_healthy = self.check_nats_connection().await;
                let status = if nats_healthy {
                    ServingStatus::Serving
                } else {
                    ServingStatus::NotServing
                };
                Ok(Response::new(HealthCheckResponse {
                    status: status as i32,
                }))
            }
            _ => Err(Status::invalid_argument(
                "Unknown service. Only `liveness` and `readiness` are supported.",
            )),
        }
    }

    type WatchStream =
        Pin<Box<dyn Stream<Item = Result<HealthCheckResponse, Status>> + Send + 'static>>;

    async fn watch(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        Err(Status::unimplemented("Watch is not implemented"))
    }
}
