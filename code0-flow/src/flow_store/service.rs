use async_trait::async_trait;
use log::{debug, error};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};
use std::sync::Arc;
use tokio::sync::Mutex;
use tucana::sagittarius::{Flow, Flows};

pub type FlowStore = Arc<Mutex<MultiplexedConnection>>;

#[derive(Debug)]
pub struct FlowStoreError {
    kind: FlowStoreErrorKind,
    flow_id: i64,
    reason: String,
}

#[derive(Debug)]
enum FlowStoreErrorKind {
    Serialization,
    RedisOperation,
}

/// Trait representing a service for managing flows in a Redis.
#[async_trait]
pub trait FlowStoreService {
    async fn new(redis_client_arc: FlowStore) -> Self;
    async fn insert_flow(&mut self, flow: Flow) -> Result<i64, FlowStoreError>;
    async fn insert_flows(&mut self, flows: Flows) -> Result<i64, FlowStoreError>;
    async fn delete_flow(&mut self, flow_id: i64) -> Result<i64, RedisError>;
    async fn delete_flows(&mut self, flow_ids: Vec<i64>) -> Result<i64, RedisError>;
    async fn get_all_flow_ids(&mut self) -> Result<Vec<i64>, RedisError>;
}

/// Struct representing a service for managing flows in a Redis.
struct FlowServiceBase {
    pub(crate) redis_client_arc: FlowStore,
}

/// Implementation of a service for managing flows in a Redis.
#[async_trait]
impl FlowStoreService for FlowServiceBase {
    async fn new(redis_client_arc: FlowStore) -> Self {
        Self { redis_client_arc }
    }

    /// Insert a list of flows into Redis
    async fn insert_flow(&mut self, flow: Flow) -> Result<i64, FlowStoreError> {
        let mut connection = self.redis_client_arc.lock().await;

        let serialized_flow = match serde_json::to_string(&flow) {
            Ok(serialized_flow) => serialized_flow,
            Err(parse_error) => {
                error!("An Error occurred {}", parse_error);
                return Err(FlowStoreError {
                    flow_id: flow.flow_id,
                    kind: FlowStoreErrorKind::Serialization,
                    reason: parse_error.to_string(),
                });
            }
        };

        let parsed_flow = connection
            .set::<String, String, i64>(flow.flow_id.to_string(), serialized_flow)
            .await;

        match parsed_flow {
            Ok(modified) => {
                debug!("Inserted flow");
                Ok(modified)
            }
            Err(redis_error) => {
                error!("An Error occurred {}", redis_error);
                Err(FlowStoreError {
                    flow_id: flow.flow_id,
                    kind: FlowStoreErrorKind::RedisOperation,
                    reason: redis_error.to_string(),
                })
            }
        }
    }

    /// Insert a flows into Redis
    async fn insert_flows(&mut self, flows: Flows) -> Result<i64, FlowStoreError> {
        let mut total_modified = 0;

        for flow in flows.flows {
            let result = self.insert_flow(flow).await?;
            total_modified += result;
        }

        Ok(total_modified)
    }

    /// Deletes a flow
    async fn delete_flow(&mut self, flow_id: i64) -> Result<i64, RedisError> {
        let mut connection = self.redis_client_arc.lock().await;
        let deleted_flow = connection.del::<i64, i64>(flow_id).await;

        match deleted_flow {
            Ok(changed_amount) => {
                debug!("{} flows where deleted", changed_amount);
                deleted_flow
            }
            Err(redis_error) => {
                error!("An Error occurred {}", redis_error);
                Err(redis_error)
            }
        }
    }

    /// Deletes a list of flows
    async fn delete_flows(&mut self, flow_ids: Vec<i64>) -> Result<i64, RedisError> {
        let mut total_modified = 0;

        for id in flow_ids {
            let result = self.delete_flow(id).await?;
            total_modified += result;
        }

        Ok(total_modified)
    }

    /// Queries for all ids in the redis
    /// Returns `Result<Vec<i64>, RedisError>`: Result of the flow ids currently in Redis
    async fn get_all_flow_ids(&mut self) -> Result<Vec<i64>, RedisError> {
        let mut connection = self.redis_client_arc.lock().await;

        let string_keys: Vec<String> = {
            match connection.keys("*").await {
                Ok(res) => res,
                Err(error) => {
                    print!("Can't retrieve keys from redis. Reason: {error}");
                    return Err(error);
                }
            }
        };

        let int_keys: Vec<i64> = string_keys
            .into_iter()
            .filter_map(|key| key.parse::<i64>().ok())
            .collect();

        Ok(int_keys)
    }
}
