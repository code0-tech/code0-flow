#[macro_export]
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

#[macro_export]
macro_rules! redis_container_test {
    ($test_name:ident, $consumer:expr) => {
        
        #[tokio::test]
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
            
            $consumer(url).await;
        }
    };
}