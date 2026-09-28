use iceoryx2::{
    node::{Node, NodeBuilder},
    prelude::{EventId, NodeName, PortName, ServiceName},
};
use iceoryx2_bb_posix::clock::Time;
use std::{sync::Arc, sync::Mutex, time::Duration};

use crate::comm_traits::{
    Communication, ListenerBuilderTrait, ListenerTrait, NotifierBuilderTrait, NotifierTrait,
    PublisherBuilderTrait, PublisherTrait, SubscriberBuilderTrait, SubscriberTrait,
};

pub type ServiceType = iceoryx2::service::ipc::Service;

static GLOBAL_NODE: Mutex<Option<Arc<Node<ServiceType>>>> = Mutex::new(None);

fn init_global_node(node_name: &str) {
    let mut node_guard = GLOBAL_NODE.lock().unwrap();
    if node_guard.is_some() {
        panic!("Double initialization of global node!");
    }
    *node_guard = Some(Arc::new(
        NodeBuilder::new()
            .name(&NodeName::new(node_name).unwrap())
            .create()
            .unwrap(),
    ));
}

pub fn unset_global_node() {
    let _ = GLOBAL_NODE.lock().unwrap().take();
}

pub fn global_node() -> Option<Arc<Node<ServiceType>>> {
    GLOBAL_NODE.lock().unwrap().clone()
}

impl PublisherTrait for iceoryx2::port::publisher::Publisher<ServiceType, [u8], Time> {
    fn send(
        &self,
        payload_size: usize,
        mut data_producer: crate::comm_traits::ProducerCallback,
    ) -> Result<bool, Box<dyn core::error::Error>> {
        let mut sample = self.loan_slice_uninit(payload_size)?;
        *sample.user_header_mut() =
            Time::now_with_clock(iceoryx2_bb_posix::clock::ClockType::Realtime).unwrap();

        data_producer(sample.payload_mut());

        unsafe { sample.assume_init() }.send()?;

        Ok(true)
    }
}

impl SubscriberTrait for iceoryx2::port::subscriber::Subscriber<ServiceType, [u8], Time> {
    fn receive(
        &self,
        mut data_consumer: crate::comm_traits::ConsumerCallback,
    ) -> Result<Option<Duration>, Box<dyn core::error::Error>> {
        if let Some(sample) = self.receive()? {
            let elapsed = sample.user_header().elapsed().unwrap();
            data_consumer(sample.payload());
            Ok(Some(elapsed))
        } else {
            Ok(None)
        }
    }
}

impl NotifierTrait for iceoryx2::port::notifier::Notifier<ServiceType> {
    fn notify(&self, id: u8) -> Result<(), Box<dyn core::error::Error>> {
        self.notify_with_custom_event_id(EventId::new(id as _))?;
        Ok(())
    }
}

impl ListenerTrait for iceoryx2::port::listener::Listener<ServiceType> {
    fn try_wait(&self) -> Result<u64, Box<dyn core::error::Error>> {
        let number_of_triggers = self.try_wait(|_| {})?;
        Ok(number_of_triggers)
    }

    fn blocking_wait(&self) -> Result<u64, Box<dyn core::error::Error>> {
        let number_of_triggers = self.blocking_wait(|_| {})?;
        Ok(number_of_triggers)
    }
}

pub struct PublisherBuilder {
    service_name: ServiceName,
    port_name: PortName,
    payload_size_hint: usize,
}

impl PublisherBuilderTrait for PublisherBuilder {
    fn new(service_name: &str, port_name: &str) -> Self {
        Self {
            service_name: ServiceName::new(service_name).unwrap(),
            port_name: PortName::new(port_name).unwrap(),
            payload_size_hint: 8,
        }
    }

    fn payload_size_hint(mut self, value: usize) -> Self {
        self.payload_size_hint = value;
        self
    }

    fn create(self) -> Result<Box<dyn PublisherTrait>, Box<dyn core::error::Error>> {
        let node = global_node().unwrap();

        let service = node
            .service_builder(&self.service_name)
            .publish_subscribe::<[u8]>()
            .user_header::<Time>()
            .open_or_create()?;

        if !self.port_name.is_empty() {
            let publisher = service
                .publisher_builder()
                .name(&self.port_name)
                .allocation_strategy(iceoryx2::prelude::AllocationStrategy::PowerOfTwo)
                .initial_max_slice_len(self.payload_size_hint)
                .create()?;
            Ok(Box::new(publisher))
        } else {
            let publisher = service
                .publisher_builder()
                .allocation_strategy(iceoryx2::prelude::AllocationStrategy::PowerOfTwo)
                .initial_max_slice_len(self.payload_size_hint)
                .create()?;
            Ok(Box::new(publisher))
        }
    }
}

pub struct SubscriberBuilder {
    service_name: ServiceName,
    port_name: PortName,
}

impl SubscriberBuilderTrait for SubscriberBuilder {
    fn new(service_name: &str, port_name: &str) -> Self {
        Self {
            service_name: ServiceName::new(service_name).unwrap(),
            port_name: PortName::new(port_name).unwrap(),
        }
    }

    fn create(self) -> Result<Box<dyn SubscriberTrait>, Box<dyn core::error::Error>> {
        let node = global_node().unwrap();

        let service = node
            .service_builder(&self.service_name)
            .publish_subscribe::<[u8]>()
            .user_header::<Time>()
            .open_or_create()?;

        if !self.port_name.is_empty() {
            let subscriber = service
                .subscriber_builder()
                .name(&self.port_name)
                .create()?;
            Ok(Box::new(subscriber))
        } else {
            let subscriber = service.subscriber_builder().create()?;
            Ok(Box::new(subscriber))
        }
    }
}

pub struct NotifierBuilder {
    service_name: ServiceName,
    port_name: PortName,
}

impl NotifierBuilderTrait for NotifierBuilder {
    fn new(service_name: &str, port_name: &str) -> Self {
        Self {
            service_name: ServiceName::new(service_name).unwrap(),
            port_name: PortName::new(port_name).unwrap(),
        }
    }

    fn create(self) -> Result<Box<dyn NotifierTrait>, Box<dyn core::error::Error>> {
        let node = global_node().unwrap();

        let service = node
            .service_builder(&self.service_name)
            .event()
            .open_or_create()?;

        if !self.port_name.is_empty() {
            let notifier = service.notifier_builder().name(&self.port_name).create()?;
            Ok(Box::new(notifier))
        } else {
            let notifier = service.notifier_builder().create()?;
            Ok(Box::new(notifier))
        }
    }
}

pub struct ListenerBuilder {
    service_name: ServiceName,
    port_name: PortName,
}

impl ListenerBuilderTrait for ListenerBuilder {
    fn new(service_name: &str, port_name: &str) -> Self {
        Self {
            service_name: ServiceName::new(service_name).unwrap(),
            port_name: PortName::new(port_name).unwrap(),
        }
    }

    fn create(self) -> Result<Box<dyn ListenerTrait>, Box<dyn core::error::Error>> {
        let node = global_node().unwrap();

        let service = node
            .service_builder(&self.service_name)
            .event()
            .open_or_create()?;

        if !self.port_name.is_empty() {
            let listener = service.listener_builder().name(&self.port_name).create()?;
            Ok(Box::new(listener))
        } else {
            let listener = service.listener_builder().create()?;
            Ok(Box::new(listener))
        }
    }
}

pub struct CommIceoryx2;

impl Communication for CommIceoryx2 {
    type Publisher = iceoryx2::port::publisher::Publisher<ServiceType, [u8], Time>;
    type Subscriber = iceoryx2::port::subscriber::Subscriber<ServiceType, [u8], Time>;
    type Listener = iceoryx2::port::listener::Listener<ServiceType>;
    type Notifier = iceoryx2::port::notifier::Notifier<ServiceType>;

    type PublisherBuilder = PublisherBuilder;
    type SubscriberBuilder = SubscriberBuilder;
    type ListenerBuilder = ListenerBuilder;
    type NotifierBuilder = NotifierBuilder;

    fn init(node_name: &str) {
        init_global_node(node_name);
    }
}
