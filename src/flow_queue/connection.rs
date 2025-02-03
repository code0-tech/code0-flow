use rabbitmq_stream_client::Environment;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type FlowQueue = Arc<Mutex<Box<Environment>>>;

pub struct RedisConfiguration {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl RedisConfiguration {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
        }
    }
}

pub async fn init_rabbitmq(redis_configuration: RedisConfiguration) -> FlowQueue {
    Arc::new(Mutex::new(Box::new(connect(redis_configuration).await)))
}

async fn connect(redis_configuration: RedisConfiguration) -> Environment {
    match Environment::builder()
        .host(&*redis_configuration.host)
        .port(redis_configuration.port)
        .username(&*redis_configuration.username)
        .password(&*redis_configuration.password)
        .build()
        .await
    {
        Ok(env) => env,
        Err(error) => panic!("Cannot connect to redis instance! Reason: {:?}", error),
    }
}
