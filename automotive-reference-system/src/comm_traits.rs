use std::{mem::MaybeUninit, time::Duration};

pub type ProducerCallback = Box<dyn FnMut(&mut [MaybeUninit<u8>])>;
pub type ConsumerCallback = Box<dyn FnMut(&[u8])>;

pub trait PublisherTrait {
    fn send(
        &self,
        payload_size: usize,
        data_producer: ProducerCallback,
    ) -> Result<bool, Box<dyn core::error::Error>>;
}

pub trait SubscriberTrait {
    fn receive(
        &self,
        data_consumer: ConsumerCallback,
    ) -> Result<Option<Duration>, Box<dyn core::error::Error>>;
}

pub trait ListenerTrait {
    fn try_wait(&self) -> Result<u64, Box<dyn core::error::Error>>;
    fn blocking_wait(&self) -> Result<u64, Box<dyn core::error::Error>>;
}

pub trait NotifierTrait {
    fn notify(&self, id: u8) -> Result<(), Box<dyn core::error::Error>>;
}

pub trait PublisherBuilderTrait {
    fn new(service_name: &str, port_name: &str) -> Self;
    fn payload_size_hint(self, value: usize) -> Self;
    fn create(self) -> Result<Box<dyn PublisherTrait>, Box<dyn core::error::Error>>;
}

pub trait SubscriberBuilderTrait {
    fn new(service_name: &str, port_name: &str) -> Self;
    fn create(self) -> Result<Box<dyn SubscriberTrait>, Box<dyn core::error::Error>>;
}

pub trait ListenerBuilderTrait {
    fn new(service_name: &str, port_name: &str) -> Self;
    fn create(self) -> Result<Box<dyn ListenerTrait>, Box<dyn core::error::Error>>;
}

pub trait NotifierBuilderTrait {
    fn new(service_name: &str, port_name: &str) -> Self;
    fn create(self) -> Result<Box<dyn NotifierTrait>, Box<dyn core::error::Error>>;
}

pub trait Communication {
    type Publisher: PublisherTrait;
    type Subscriber: SubscriberTrait;
    type Listener: ListenerTrait;
    type Notifier: NotifierTrait;

    type PublisherBuilder: PublisherBuilderTrait;
    type SubscriberBuilder: SubscriberBuilderTrait;
    type ListenerBuilder: ListenerBuilderTrait;
    type NotifierBuilder: NotifierBuilderTrait;

    fn init(node_name: &str);
}
