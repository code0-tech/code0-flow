use crate::flow_store::connection::FlowStore;
use async_trait::async_trait;
use log::error;
use redis::{AsyncCommands, RedisError, RedisResult};
use tucana::sagittarius::{Flow, Flows};

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
pub trait FlowStoreServiceBase {
    async fn new(redis_client_arc: FlowStore) -> Self;
    async fn insert_flow(&mut self, flow: Flow) -> Result<i64, FlowStoreError>;
    async fn insert_flows(&mut self, flows: Flows) -> Result<i64, FlowStoreError>;
    async fn delete_flow(&mut self, flow_id: i64) -> Result<i64, RedisError>;
    async fn delete_flows(&mut self, flow_ids: Vec<i64>) -> Result<i64, RedisError>;
    async fn get_all_flow_ids(&mut self) -> Result<Vec<i64>, RedisError>;
}

/// Struct representing a service for managing flows in a Redis.
#[derive(Clone)]
pub struct FlowStoreService {
    pub(crate) redis_client_arc: FlowStore,
}

/// Implementation of a service for managing flows in a Redis.
#[async_trait]
impl FlowStoreServiceBase for FlowStoreService {
    async fn new(redis_client_arc: FlowStore) -> FlowStoreService {
        FlowStoreService { redis_client_arc }
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

        let insert_result: RedisResult<()> = connection.set(flow.flow_id, serialized_flow).await;

        match insert_result {
            Err(redis_error) => {
                error!("An Error occurred {}", redis_error);
                Err(FlowStoreError {
                    flow_id: flow.flow_id,
                    kind: FlowStoreErrorKind::RedisOperation,
                    reason: redis_error.to_string(),
                })
            }
            _ => Ok(1),
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
        let deleted_flow: RedisResult<i64> = connection.del(flow_id).await;

        match deleted_flow {
            Ok(int) => Ok(int),
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

#[cfg(test)]
mod tests {
    use crate::flow_store::connection::create_flow_store_connection;
    use crate::flow_store::connection::FlowStore;
    use crate::flow_store::service::FlowStoreService;
    use crate::flow_store::service::FlowStoreServiceBase;
    use redis::AsyncCommands;
    use serial_test::serial;
    use testcontainers::core::IntoContainerPort;
    use testcontainers::core::WaitFor;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::GenericImage;
    use tucana::sagittarius::{Flow, Flows};

    macro_rules! redis_integration_test {
        ($test_name:ident, $consumer:expr) => {
            #[tokio::test]
            #[serial]
            async fn $test_name() {
                let port: u16 = 6379;
                let image_name = "redis";
                let wait_message = "Ready to accept connections";

                let container = GenericImage::new(image_name, "latest")
                    .with_exposed_port(port.tcp())
                    .with_wait_for(WaitFor::message_on_stdout(wait_message))
                    .start()
                    .await
                    .unwrap();

                let host = container.get_host().await.unwrap();
                let host_port = container.get_host_port_ipv4(port).await.unwrap();
                let url = format!("redis://{host}:{host_port}");
                println!("Redis server started correctly on: {}", url.clone());

                let connection = create_flow_store_connection(url).await;
                let base = FlowStoreService::new(connection.clone()).await;

                $consumer(connection, base).await;
                let _ = container.stop().await;
            }
        };
    }

    redis_integration_test!(
        insert_one_flow,
        (|connection: FlowStore, mut service: FlowStoreService| async move {
            let flow = Flow {
                flow_id: 1,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            match service.insert_flow(flow.clone()).await {
                Ok(i) => println!("{}", i),
                Err(err) => println!("{}", err.reason),
            };

            let redis_result: Option<String> = {
                let mut redis_cmd = connection.lock().await;
                redis_cmd.get("1").await.unwrap()
            };

            println!("{}", redis_result.clone().unwrap());

            assert!(redis_result.is_some());
            let redis_flow: Flow = serde_json::from_str(&*redis_result.unwrap()).unwrap();
            assert_eq!(redis_flow, flow);
        })
    );

    redis_integration_test!(
        insert_will_overwrite_existing_flow,
        (|connection: FlowStore, mut service: FlowStoreService| async move {
            let flow = Flow {
                flow_id: 1,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            match service.insert_flow(flow.clone()).await {
                Ok(i) => println!("{}", i),
                Err(err) => println!("{}", err.reason),
            };
            
            let flow_overwrite = Flow {
                flow_id: 1,
                r#type: "ABC".to_string(),
                settings: vec![],
                starting_node: None,
            };
            
            let _ = service.insert_flow(flow_overwrite).await;
            let amount = service.get_all_flow_ids().await;
            assert_eq!(amount.unwrap().len(), 1);
            
           let redis_result: Option<String> = {
                let mut redis_cmd = connection.lock().await;
                redis_cmd.get("1").await.unwrap()
            };

            println!("{}", redis_result.clone().unwrap());

            assert!(redis_result.is_some());
            let redis_flow: Flow = serde_json::from_str(&*redis_result.unwrap()).unwrap();
            assert_eq!(redis_flow.r#type, "ABC".to_string());
        }) 
    );

    redis_integration_test!(
        insert_many_flows,
        (|_connection: FlowStore, mut service: FlowStoreService| async move {
            let flow_one = Flow {
                flow_id: 1,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_two = Flow {
                flow_id: 2,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_three = Flow {
                flow_id: 3,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_vec = vec![flow_one.clone(), flow_two.clone(), flow_three.clone()];
            let flows = Flows { flows: flow_vec };

            let amount = service.insert_flows(flows).await.unwrap();
            assert_eq!(amount, 3);
        })
    );

    redis_integration_test!(
        delete_one_existing_flow,
        (|connection: FlowStore, mut service: FlowStoreService| async move {
            let flow = Flow {
                flow_id: 1,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            match service.insert_flow(flow.clone()).await {
                Ok(i) => println!("{}", i),
                Err(err) => println!("{}", err.reason),
            };

            let result = service.delete_flow(1).await;

            assert_eq!(result.unwrap(), 1);

            let redis_result: Option<String> = {
                let mut redis_cmd = connection.lock().await;
                redis_cmd.get("1").await.unwrap()
            };

            assert!(redis_result.is_none());
        })
    );

    redis_integration_test!(
        delete_one_non_existing_flow,
        (|_connection: FlowStore, mut service: FlowStoreService| async move {
            let result = service.delete_flow(1).await;
            assert_eq!(result.unwrap(), 0);
        })
    );

    redis_integration_test!(
        delete_many_existing_flows,
        (|_connection: FlowStore, mut service: FlowStoreService| async move {
            let flow_one = Flow {
                flow_id: 1,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_two = Flow {
                flow_id: 2,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_three = Flow {
                flow_id: 3,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_vec = vec![flow_one.clone(), flow_two.clone(), flow_three.clone()];
            let flows = Flows { flows: flow_vec };

            let amount = service.insert_flows(flows).await.unwrap();
            assert_eq!(amount, 3);
            
            let deleted_amount = service.delete_flows(vec![1, 2, 3]).await;
            assert_eq!(deleted_amount.unwrap(), 3);
        })
    );

    redis_integration_test!(
        delete_many_non_existing_flows,
        (|_connection: FlowStore, mut service: FlowStoreService| async move {
            let deleted_amount = service.delete_flows(vec![1, 2, 3]).await;
            assert_eq!(deleted_amount.unwrap(), 0);
        })
    );

    redis_integration_test!(
        get_existing_flow_ids,
        (|_connection: FlowStore, mut service: FlowStoreService| async move {
            let flow_one = Flow {
                flow_id: 1,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_two = Flow {
                flow_id: 2,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_three = Flow {
                flow_id: 3,
                r#type: "".to_string(),
                settings: vec![],
                starting_node: None,
            };

            let flow_vec = vec![flow_one.clone(), flow_two.clone(), flow_three.clone()];
            let flows = Flows { flows: flow_vec };

            let amount = service.insert_flows(flows).await.unwrap();
            assert_eq!(amount, 3);
            
            let mut flow_ids = service.get_all_flow_ids().await.unwrap();
            flow_ids.sort();
            
            assert_eq!(flow_ids, vec![1, 2, 3]);
        })
    );

    redis_integration_test!(
        get_empty_flow_ids,
        (|_connection: FlowStore, mut service: FlowStoreService| async move {
            let flow_ids = service.get_all_flow_ids().await;
            assert_eq!(flow_ids.unwrap(), Vec::<i64>::new());
        })
    );

}
