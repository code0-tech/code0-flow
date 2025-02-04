use crate::flow_queue::connection::{get_flow_channel, FlowChannel, FlowQueue};
use crate::flow_queue::delegate::{Delegate, QueueDelegate};
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

/// Sends a message into a queue
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

/// Consumes a message
///
/// Creates a delegate that waits on messages and consumes them.
///
/// # Params
/// - channel: FlowChannel of the send message
/// - queue_name: Name of the Queue that should be listened to
/// - delegate: Consumer delegate of the message
///
/// # Example
/// ```
/// use lapin::message::Delivery;
/// use code0_flow::flow_queue::delegate::Delegate;
/// use code0_flow::flow_queue::connection::get_flow_channel;
/// use code0_flow::flow_queue::name::{QueueName, QueuePrefix, QueueProtocol};
/// use code0_flow::flow_queue::handler::consume_message;
///
/// struct HttpDelegate;
///
/// impl Delegate for HttpDelegate {
///     fn handle_delivery(&self, delivery: Delivery) {
///         todo!("Handle delivery!")
///     }
/// }
///
/// async fn main() {
///     let uri = "abc";
///     let channel = get_flow_channel(uri).await;
///     let queue_name = QueueName {
///         prefix: QueuePrefix::Send,
///         protocol: QueueProtocol::Rest,
///     };
///
///     consume_message(channel, queue_name, HttpDelegate).await;
/// }
/// ```
pub async fn consume_message<T: Delegate>(
    channel: FlowChannel,
    queue_name: QueueName,
    delegate: T,
) {
    let name = queue_name.prefix + queue_name.protocol;
    let channel_arc = channel.lock().await;

    let mut consumer = channel_arc
        .basic_consume(
            &*name,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    consumer.set_delegate(QueueDelegate { delegate });
}
