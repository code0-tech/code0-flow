use lapin::{Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type FlowQueue = Arc<Mutex<Box<Connection>>>;

pub type FlowChannel = Arc<Mutex<Box<Channel>>>;

pub async fn connect(uri: &str) -> Connection {
    match Connection::connect(uri, ConnectionProperties::default()).await {
        Ok(env) => env,
        Err(error) => panic!("Cannot connect to redis instance! Reason: {:?}", error),
    }
}

pub async fn get_flow_channel(uri: &str) -> FlowChannel {
    let connection = connect(uri).await;

    match connection.create_channel().await {
        Ok(channel) => Arc::new(Mutex::new(Box::new(channel))),
        Err(error) => panic!("Cannot create channel {:?}", error),
    }
}

#[cfg(test)]
mod tests {
    use crate::flow_queue::connection::connect;
    use testcontainers::core::{IntoContainerPort, WaitFor};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::GenericImage;
    use crate::rabbitmq_container_test;

    rabbitmq_container_test!(test_rabbitmq_startup, (|url: String| async move {
        println!("RabbitMQ started with the url: {}", url);
    }));  
    
    rabbitmq_container_test!(test_rabbitmq_connection, (|url: String| async move {
        connect(&*url).await;
    }));

}
