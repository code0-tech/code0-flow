use lapin::message::{Delivery, DeliveryResult};
use lapin::ConsumerDelegate;
use log::debug;
use std::future::Future;
use std::pin::Pin;

/// Delegate trait to implement.
///
/// Use as delegate for RabbitMQ
///
/// # Example
/// ```
/// use lapin::message::Delivery;
/// use code0_flow::flow_queue::delegate::Delegate;
///
/// struct HttpDelegate;
///
/// impl Delegate for HttpDelegate {
///     fn handle_delivery(&self, delivery: Delivery) {
///         todo!("Handle delivery!")
///     }
/// }
/// ```
pub trait Delegate {
    fn handle_delivery(&self, delivery: Delivery);
}

pub struct QueueDelegate<T: Delegate> {
    pub delegate: T,
}

impl<T: Delegate> QueueDelegate<T> {
    pub fn new(delegate: T) -> Self {
        QueueDelegate { delegate }
    }

    pub fn deliver(&self, delivery: Delivery) {
        self.delegate.handle_delivery(delivery);
    }
}

impl<T: Delegate> ConsumerDelegate for QueueDelegate<T> {
    fn on_new_delivery(
        &self,
        delivery: DeliveryResult,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        async move {
            let optional_delivery = match delivery {
                Ok(option) => option,
                Err(_) => return,
            };
            let delivery = match optional_delivery {
                Some(del) => del,
                None => return,
            };

            self.delegate.handle_delivery(delivery);
        }
    }

    fn drop_prefetched_messages(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let future = async move {
            debug!("Dropping prefetched messages...");
        };

        Box::pin(future)
    }
}
