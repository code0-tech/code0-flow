use lapin::{Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type FlowQueue = Arc<Mutex<Box<Connection>>>;

pub type FlowChannel = Arc<Mutex<Box<Channel>>>;

async fn build_connection(rabbitmq_url: &str) -> Connection {
    match Connection::connect(rabbitmq_url, ConnectionProperties::default()).await {
        Ok(env) => env,
        Err(error) => panic!("Cannot connect to FlowQueue (RabbitMQ) instance! Reason: {:?}", error),
    }
}

pub async fn create_flow_channel_connection(uri: &str) -> FlowChannel {
    let connection = build_connection(uri).await;

    match connection.create_channel().await {
        Ok(channel) => Arc::new(Mutex::new(Box::new(channel))),
        Err(error) => panic!("Cannot create channel {:?}", error),
    }
}

#[cfg(test)]
mod tests {
    use testcontainers::core::{IntoContainerPort, WaitFor};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::GenericImage;
    use crate::flow_queue::connection::build_connection;

    macro_rules! rabbitmq_container_test {
        ($test_name:ident, $consumer:expr) => {
            
            #[tokio::test]
            async fn $test_name() {
                let port: u16 = 5672;
                let image_name = "rabbitmq";
                let wait_message = "Server startup complete";
    
                let container = GenericImage::new(image_name, "latest")
                    .with_exposed_port(port.tcp())
                    .with_wait_for(WaitFor::message_on_stdout(wait_message))
                    .start()
                    .await
                    .unwrap();
    
                let host_port = container.get_host_port_ipv4(port).await.unwrap();
                let url = format!("amqp://guest:guest@localhost:{}", host_port);
                
                $consumer(url).await;
            }
        };
    }

    rabbitmq_container_test!(test_rabbitmq_startup, (|url: String| async move {
        println!("RabbitMQ started with the url: {}", url);
    }));  
    
    rabbitmq_container_test!(test_rabbitmq_connection, (|url: String| async move {
        build_connection(&*url).await;
    }));

}
