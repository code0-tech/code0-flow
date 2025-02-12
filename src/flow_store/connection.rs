use redis::Client;

pub fn build_connection(redis_url: String) -> Client {
    match Client::open(redis_url) {
        Ok(client) => client,
        Err(con_error) => panic!("{}", con_error),
    }
}

#[cfg(test)]
mod tests {
    use testcontainers::core::IntoContainerPort;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::GenericImage;
    use testcontainers::core::WaitFor;
    use crate::flow_store::connection::build_connection;
    use crate::redis_container_test;
    
    redis_container_test!(test_redis_startup, (|url: String| async move {
        println!("Redis server started correctly on: {}", url);
    }));
    
    redis_container_test!(test_redis_connection, (|url: String| async move {
        let result = build_connection(url).get_connection();
        assert!(result.is_ok());
    }));
}
