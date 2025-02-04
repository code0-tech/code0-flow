use crate::flow_queue::connection::{get_flow_channel, FlowChannel, FlowQueue};
use crate::flow_queue::name::{QueueName, QueuePrefix, QueueProtocol};
use lapin::message::{Delivery, DeliveryResult};
use lapin::options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions};
use lapin::protocol::basic::gen_return;
use lapin::types::FieldTable;
use lapin::{ConsumerDelegate, Error};
use log::{debug, error, info};
use std::future::Future;
use std::pin::Pin;

/// # Declares all given queues
///
/// ## Expected behavior
/// If a queue cannot be created, the services stops
pub async fn declare_queues(flow_channel: FlowChannel, names: Vec<QueueName>) {
    let channel_arc = flow_channel.lock().await;
    for name in names {
        let channel_name = name.prefix + name.protocol;
        let queue_result = channel_arc
            .queue_declare(
                &*channel_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;

        match queue_result {
            Ok(_) => {
                info!("Declared queue: {}", channel_name)
            }
            Err(error) => {
                panic!("Cannot declare queue: {}, Reason: {}", channel_name, error);
            }
        };
    }
}

pub async fn send_message(
    flow_channel: FlowChannel,
    queue_name: QueueName,
    payload: &Vec<u8>,
) -> Result<bool, Error> {
    let channel_arc = flow_channel.lock().await;
    let name = queue_name.prefix + queue_name.protocol;

    let result = channel_arc
        .basic_publish(
            "",
            &*name,
            BasicPublishOptions::default(),
            payload,
            Default::default(),
        )
        .await;

    match result {
        Ok(_) => Ok(true),
        Err(error) => Err(error),
    }
}

pub async fn consume_message(channel: FlowChannel, queue_protocol: QueueProtocol) {
    let name = QueuePrefix::Send + queue_protocol;
    let channel_arc = channel.lock().await;

    let mut consumer = channel_arc
        .basic_consume(
            &*name,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    consumer.set_delegate(SendQueueDelegate);
}