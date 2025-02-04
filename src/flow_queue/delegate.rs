use lapin::message::{Delivery, DeliveryResult};
use lapin::ConsumerDelegate;
use log::debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Delegate trait to implement.
pub trait Delegate: Sync + Send {
    fn handle_delivery(&self, delivery: Delivery);
}

pub struct QueueDelegate<T: Delegate> {
    pub delegate: Arc<T>, // Use Arc for safe ownership transfer
}

impl<T: Delegate> QueueDelegate<T> {
    pub fn new(delegate: T) -> Self {
        QueueDelegate {
            delegate: Arc::new(delegate),
        }
    }

    pub fn deliver(&self, delivery: Delivery) {
        self.delegate.handle_delivery(delivery);
    }
}

impl<T: Delegate + Send + Sync + 'static> ConsumerDelegate for QueueDelegate<T> {
    fn on_new_delivery(
        &self,
        delivery: DeliveryResult,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let delegate = Arc::clone(&self.delegate);

        Box::pin(async move {
            let optional_delivery = match delivery {
                Ok(option) => option,
                Err(_) => return,
            };
            let delivery = match optional_delivery {
                Some(del) => del,
                None => return,
            };

            delegate.handle_delivery(delivery); // Use cloned delegate
        })
    }

    fn drop_prefetched_messages(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(async move {
            debug!("Dropping prefetched messages...");
        })
    }
}
