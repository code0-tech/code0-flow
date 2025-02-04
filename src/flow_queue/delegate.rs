use lapin::message::{DeliveryResult};
use lapin::ConsumerDelegate;
use std::future::Future;
use std::pin::Pin;

struct QueueDelegate;

impl ConsumerDelegate for QueueDelegate {
    fn on_new_delivery(
        &self,
        delivery: DeliveryResult,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let optional_delivery = match delivery {
            Ok(option) => option,
            Err(error) => {
                todo!("error handling")
            }
        };

        let delivery = match optional_delivery {
            Some(del) => del,
            None => {
                todo!("error handling")
            }
        };
        todo!("consumer shoud consume the data of delivy as &Vec<u8>")
    }

    fn drop_prefetched_messages(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!("")
    }
}
