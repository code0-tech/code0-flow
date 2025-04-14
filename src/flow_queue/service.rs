use std::{sync::Arc, time::Duration};

use futures_lite::StreamExt;
use lapin::{options::QueueDeclareOptions, types::FieldTable, Channel};
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::connection::build_connection;

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    ExecuteFlow,
    TestExecuteFlow,
}

#[derive(Serialize, Deserialize)]
pub struct Sender {
    pub name: String,
    pub protocol: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub sender: Sender,
    pub timestamp: i64,
    pub message_id: String,
    pub body: String,
}

pub struct RabbitmqClient {
    pub channel: Arc<Mutex<Channel>>,
}

#[derive(Debug)]
pub enum RabbitMqError {
    LapinError(lapin::Error),
    ConnectionError(String),
    TimeoutError,
    DeserializationError,
}

impl From<lapin::Error> for RabbitMqError {
    fn from(error: lapin::Error) -> Self {
        RabbitMqError::LapinError(error)
    }
}

impl From<std::io::Error> for RabbitMqError {
    fn from(error: std::io::Error) -> Self {
        RabbitMqError::ConnectionError(error.to_string())
    }
}

impl std::fmt::Display for RabbitMqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RabbitMqError::LapinError(err) => write!(f, "RabbitMQ error: {}", err),
            RabbitMqError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            RabbitMqError::TimeoutError => write!(f, "Operation timed out"),
            RabbitMqError::DeserializationError => write!(f, "Failed to deserialize message"),
        }
    }
}

impl RabbitmqClient {
    // Create a new RabbitMQ client with channel
    pub async fn new(rabbitmq_url: &str) -> Self {
        let connection = build_connection(rabbitmq_url).await;
        let channel = connection.create_channel().await.unwrap();

        match channel
            .queue_declare(
                "send_queue",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(_) => (),
            Err(err) => log::error!("Failed to declare send_queue: {}", err),
        }

        match channel
            .queue_declare(
                "recieve_queue",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(_) => (),
            Err(err) => log::error!("Failed to declare recieve_queue: {}", err),
        }

        RabbitmqClient {
            channel: Arc::new(Mutex::new(channel)),
        }
    }

    // Send message to the queue
    pub async fn send_message(
        &self,
        message_json: String,
        queue_name: &str,
    ) -> Result<(), lapin::Error> {
        let channel = self.channel.lock().await;

        channel
            .basic_publish(
                "",         // exchange
                queue_name, // routing key (queue name)
                lapin::options::BasicPublishOptions::default(),
                message_json.as_bytes(),
                lapin::BasicProperties::default(),
            )
            .await?;

        Ok(())
    }

    // Receive messages from a queue
    // Receive messages from a queue with no timeout
    pub async fn await_message_no_timeout(
        &self,
        queue_name: &str,
        message_id: String,
        ack_on_success: bool,
    ) -> Result<Message, RabbitMqError> {
        let mut consumer = {
            let channel = self.channel.lock().await;

            let consumer_res = channel
                .basic_consume(
                    queue_name,
                    "consumer",
                    lapin::options::BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await;

            match consumer_res {
                Ok(consumer) => consumer,
                Err(err) => panic!("{}", err),
            }
        };

        debug!("Starting to consume from {}", queue_name);

        while let Some(delivery_result) = consumer.next().await {
            let delivery = match delivery_result {
                Ok(del) => del,
                Err(_) => return Err(RabbitMqError::DeserializationError),
            };
            let data = &delivery.data;
            let message_str = match std::str::from_utf8(&data) {
                Ok(str) => str,
                Err(_) => {
                    return Err(RabbitMqError::DeserializationError);
                }
            };

            debug!("Received message: {}", message_str);

            // Parse the message
            let message = match serde_json::from_str::<Message>(message_str) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to parse message: {}", e);
                    return Err(RabbitMqError::DeserializationError);
                }
            };

            if message.message_id == message_id {
                if ack_on_success {
                    delivery
                        .ack(lapin::options::BasicAckOptions::default())
                        .await
                        .expect("Failed to acknowledge message");
                }

                return Ok(message);
            }
        }
        Err(RabbitMqError::DeserializationError)
    }

    // Function intended to get used by the runtime
    pub async fn consume_message(
        &self,
        queue_name: &str,
        ack_on_success: bool,
    ) -> Result<Message, RabbitMqError> {
        let mut consumer = {
            let channel = self.channel.lock().await;

            let consumer_res = channel
                .basic_consume(
                    queue_name,
                    "consumer",
                    lapin::options::BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await;

            match consumer_res {
                Ok(consumer) => consumer,
                Err(err) => panic!("{}", err),
            }
        };

        debug!("Starting to consume from {}", queue_name);

        while let Some(delivery_result) = consumer.next().await {
            let delivery = match delivery_result {
                Ok(del) => del,
                Err(_) => return Err(RabbitMqError::DeserializationError),
            };
            let data = &delivery.data;
            let message_str = match std::str::from_utf8(&data) {
                Ok(str) => str,
                Err(_) => {
                    return Err(RabbitMqError::DeserializationError);
                }
            };

            debug!("Received message: {}", message_str);

            // Parse the message
            let message = match serde_json::from_str::<Message>(message_str) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to parse message: {}", e);
                    return Err(RabbitMqError::DeserializationError);
                }
            };

            if ack_on_success {
                delivery
                    .ack(lapin::options::BasicAckOptions::default())
                    .await
                    .expect("Failed to acknowledge message");
            }

            return Ok(message);
        }
        Err(RabbitMqError::DeserializationError)
    }

    // Receive messages from a queue with timeout
    pub async fn await_message(
        &self,
        queue_name: &str,
        message_id: String,
        timeout: Duration,
        ack_on_success: bool,
    ) -> Result<Message, RabbitMqError> {
        // Set a timeout
        match tokio::time::timeout(
            timeout,
            self.await_message_no_timeout(queue_name, message_id, ack_on_success),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                debug!(
                    "Timeout waiting for message after {} seconds",
                    timeout.as_secs()
                );
                Err(RabbitMqError::TimeoutError)
            }
        }
    }
}
