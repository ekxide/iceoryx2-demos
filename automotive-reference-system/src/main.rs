use clap::{Parser, Subcommand};
use comm::{
    comm_iceoryx2::{CommIceoryx2, GLOBAL_NODE},
    comm_traits::{
        Communication, ListenerBuilderTrait, ListenerTrait, NotifierBuilderTrait, NotifierTrait,
        PublisherBuilderTrait, PublisherTrait, SubscriberBuilderTrait, SubscriberTrait,
    },
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::{fmt::Display, mem::MaybeUninit, time::Duration, time::Instant};

const PAYLOAD_SIZE: usize = 128;

#[derive(Debug)]
struct Latency {
    min: Duration,
    max: Duration,
    cur: Duration,
    avg: Duration,
    run: u32,
}

impl Display for Latency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min != Duration::MAX {
            write!(
                f,
                "min: {:.2}us, max: {:.2}us, avg: {:.2}us",
                self.min.as_nanos() as f64 / 1000.0,
                self.max.as_nanos() as f64 / 1000.0,
                self.avg.as_nanos() as f64 / 1000.0
            )
        } else {
            write!(f, ".")
        }
    }
}

impl Latency {
    fn new() -> Self {
        Self {
            min: Duration::MAX,
            max: Duration::ZERO,
            cur: Duration::ZERO,
            avg: Duration::ZERO,
            run: 0,
        }
    }

    fn set(&mut self, value: Duration) {
        self.run += 1;
        if self.run > 100 {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
            self.cur = value;
            self.avg = ((self.run - 1) * self.avg + value) / self.run;
        }
    }

    fn reset(&mut self) {
        self.cur = Duration::ZERO
    }
}

#[derive(Subcommand, Clone)]
enum NodeType {
    Actor,
    Sensor,
    Processing {
        #[arg(short, long)]
        out_services: Vec<String>,
        #[arg(long, help = "The name of outgoing port")]
        out_ports: Vec<String>,
        #[arg(long, help = "The name of outgoing notifier port")]
        out_ports_event: Vec<String>,
    },
}

#[derive(Parser, Clone)]
struct CliArgs {
    #[arg(short, long)]
    interval_in_ms: u64,
    #[arg(short, long, default_value = "2")]
    cpu_load: u64,
    #[arg(short, long, default_value = "false")]
    event_triggering: bool,
    #[arg(short, long, default_value = "")]
    node_name: String,
    #[arg(long, help = "The name of publisher/subscriber port")]
    port_names: Vec<String>,
    #[arg(long, help = "The name of notifier/listener port")]
    port_names_event: Vec<String>,
    #[arg(short, long)]
    services: Vec<String>,
    #[command(subcommand)]
    node: NodeType,
}

struct Sensor {
    publishers: Vec<Box<dyn PublisherTrait>>,
    notifiers: Vec<Box<dyn NotifierTrait>>,
    interval_in_ms: u64,
    cpu_load: u64,
    event_triggering: bool,
}

fn produce_data(raw_buffer: &mut [MaybeUninit<u8>]) {
    unsafe { core::ptr::write_bytes(raw_buffer.as_mut_ptr() as *mut u8, 0, raw_buffer.len()) };
}

// worker thread to simulate CPU load
fn worker(percentage: u32, terminate_flag: Arc<AtomicBool>) {
    let spin_duration = Duration::from_millis((percentage as u64 * 10) as u64);
    let sleep_duration = Duration::from_millis((100 - percentage as u64) * 10);

    while !terminate_flag.load(Ordering::Relaxed) {
        let spin_start = Instant::now();
        while spin_start.elapsed() < spin_duration {
            // Spin
        }
        thread::sleep(sleep_duration);
    }
}

fn create_endpoint_with_retry<T, F>(endpoint_creator: F) -> Box<T>
where
    T: ?Sized,
    F: Fn() -> Result<Box<T>, Box<dyn core::error::Error>>,
{
    let mut remaining_retries = 100;
    loop {
        match endpoint_creator() {
            Ok(endpoint) => {
                break endpoint;
            }
            Err(e) => {
                if remaining_retries == 0 {
                    panic!("Could not create Endpoint! Error: {e}");
                }
                remaining_retries -= 1;
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}

impl Sensor {
    fn new<C: Communication>(
        node_name: String,
        services: Vec<String>,
        port_names: Vec<String>,
        port_names_event: Vec<String>,
        interval_in_ms: u64,
        cpu_load: u64,
        event_triggering: bool,
    ) -> Self {
        C::init(&node_name);

        let mut publishers = vec![];
        let mut notifiers = vec![];
        let mut port_names_iter = port_names.iter();
        let mut port_names_event_iter = port_names_event.iter();

        for service in services.iter() {
            let port_name = port_names_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            publishers.push(create_endpoint_with_retry(|| {
                <C::PublisherBuilder as PublisherBuilderTrait>::new(&service, &port_name.as_str())
                    .payload_size_hint(PAYLOAD_SIZE)
                    .create()
            }));
        }

        for service in services.iter() {
            let port_name = port_names_event_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            notifiers.push(create_endpoint_with_retry(|| {
                <C::NotifierBuilder as NotifierBuilderTrait>::new(&service, &port_name.as_str())
                    .create()
            }))
        }

        Sensor {
            publishers,
            notifiers,
            interval_in_ms,
            cpu_load,
            event_triggering,
        }
    }

    fn main_loop(&self) {
        let node = GLOBAL_NODE.get().unwrap();

        let terminate_flag = Arc::new(AtomicBool::new(false));
        let mut counter = 0u64;
        let num_cores = 1;
        let percentage = self.cpu_load as u32;

        let handles: Vec<_> = (0..num_cores)
            .map(|_| {
                let terminate_flag = terminate_flag.clone();
                thread::spawn(move || {
                    worker(percentage, terminate_flag);
                })
            })
            .collect();

        while node
            .wait(Duration::from_millis(self.interval_in_ms))
            .is_ok()
        {
            for (i, publisher) in self.publishers.iter().enumerate() {
                publisher
                    .send(PAYLOAD_SIZE, Box::new(produce_data))
                    .unwrap();
                if self.event_triggering {
                    self.notifiers[i].notify(0).unwrap();
                }
            }

            println!("send iteration: {}", counter);
            counter += 1;
            std::thread::sleep(Duration::from_millis(self.interval_in_ms));
        }

        // wrap up and terminate threads
        terminate_flag.store(true, Ordering::Relaxed);
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

struct Actor {
    subscribers: Vec<Box<dyn SubscriberTrait>>,
    listeners: Vec<Box<dyn ListenerTrait>>,
    samples: Vec<bool>,
    interval_in_ms: u64,
    cpu_load: u64,
    event_triggering: bool,
}

impl Actor {
    fn new<C: Communication>(
        node_name: String,
        services: Vec<String>,
        port_names: Vec<String>,
        port_names_event: Vec<String>,
        interval_in_ms: u64,
        cpu_load: u64,
        event_triggering: bool,
    ) -> Self {
        C::init(&node_name);

        let mut subscribers = vec![];
        let mut listeners = vec![];
        let mut samples = vec![];
        let mut port_names_iter = port_names.iter();
        let mut port_names_event_iter = port_names_event.iter();

        for service in services.iter() {
            let port_name = port_names_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            subscribers.push(create_endpoint_with_retry(|| {
                <C::SubscriberBuilder as SubscriberBuilderTrait>::new(&service, &port_name).create()
            }));
            samples.push(false);
        }

        for service in services.iter() {
            let port_name = port_names_event_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            listeners.push(create_endpoint_with_retry(|| {
                <C::ListenerBuilder as ListenerBuilderTrait>::new(&service, &port_name).create()
            }));
        }

        Self {
            subscribers,
            listeners,
            samples,
            interval_in_ms,
            cpu_load,
            event_triggering,
        }
    }

    fn main_loop(&mut self) {
        let node = GLOBAL_NODE.get().unwrap();
        let mut latency = Latency::new();
        let num_cores = 1;
        let percentage = self.cpu_load as u32;
        let terminate_flag = Arc::new(AtomicBool::new(false));

        let handles: Vec<_> = (0..num_cores)
            .map(|_| {
                let terminate_flag = terminate_flag.clone();
                thread::spawn(move || {
                    worker(percentage, terminate_flag);
                })
            })
            .collect();

        while node
            .wait(Duration::from_millis(self.interval_in_ms))
            .is_ok()
        {
            if self.event_triggering {
                for (i, sample) in self.samples.iter().enumerate() {
                    if !sample && self.listeners[i].blocking_wait().is_err() {
                        return;
                    }
                    while self.listeners[i].try_wait().unwrap() != 0 {}
                }
            }

            for (i, subscriber) in self.subscribers.iter().enumerate() {
                while let Some(l) = subscriber.receive(Box::new(|_| {})).unwrap() {
                    latency.set(l);
                    self.samples[i] = true;
                }
            }

            let mut has_all_samples = true;
            for sample in &self.samples {
                if !sample {
                    has_all_samples = false;
                    break;
                }
            }

            if has_all_samples {
                println!("latency: {}", latency);
                latency.reset();
                for sample in &mut self.samples {
                    *sample = false;
                }
            }
            std::thread::sleep(Duration::from_millis(self.interval_in_ms));
        }

        // wrap up and terminate threads
        terminate_flag.store(true, Ordering::Relaxed);
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

struct Processor {
    publishers: Vec<Box<dyn PublisherTrait>>,
    subscribers: Vec<Box<dyn SubscriberTrait>>,
    notifiers: Vec<Box<dyn NotifierTrait>>,
    listeners: Vec<Box<dyn ListenerTrait>>,
    samples: Vec<bool>,
    interval_in_ms: u64,
    cpu_load: u64,
    event_triggering: bool,
}

impl Processor {
    fn new<C: Communication>(
        node_name: String,
        services_in: Vec<String>,
        services_out: Vec<String>,
        port_names_in: Vec<String>,
        port_names_out: Vec<String>,
        port_names_in_event: Vec<String>,
        port_names_out_event: Vec<String>,
        interval_in_ms: u64,
        cpu_load: u64,
        event_triggering: bool,
    ) -> Self {
        C::init(&node_name);

        let mut publishers = vec![];
        let mut subscribers = vec![];
        let mut notifiers = vec![];
        let mut listeners = vec![];
        let mut samples = vec![];
        let mut port_names_in_iter = port_names_in.iter();
        let mut port_names_out_iter = port_names_out.iter();
        let mut port_names_in_event_iter = port_names_in_event.iter();
        let mut port_names_out_event_iter = port_names_out_event.iter();

        for service in services_in.iter() {
            let port_name = port_names_in_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            subscribers.push(create_endpoint_with_retry(|| {
                <C::SubscriberBuilder as SubscriberBuilderTrait>::new(&service, &port_name).create()
            }));

            samples.push(false);
        }

        for service in services_in.iter() {
            let port_name = port_names_in_event_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            listeners.push(create_endpoint_with_retry(|| {
                <C::ListenerBuilder as ListenerBuilderTrait>::new(&service, &port_name).create()
            }));
        }

        for service in services_out.iter() {
            let port_name = port_names_out_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            publishers.push(create_endpoint_with_retry(|| {
                <C::PublisherBuilder as PublisherBuilderTrait>::new(&service, &port_name)
                    .payload_size_hint(PAYLOAD_SIZE)
                    .create()
            }));
        }

        for service in services_out.iter() {
            let port_name = port_names_out_event_iter
                .next()
                .cloned()
                .unwrap_or_else(|| String::new());
            notifiers.push(create_endpoint_with_retry(|| {
                <C::NotifierBuilder as NotifierBuilderTrait>::new(&service, &port_name).create()
            }));
        }

        Self {
            publishers,
            subscribers,
            notifiers,
            listeners,
            samples,
            interval_in_ms,
            cpu_load,
            event_triggering,
        }
    }

    fn main_loop(&mut self) {
        let node = GLOBAL_NODE.get().unwrap();
        let mut latency = Latency::new();
        let num_cores = 1;
        let percentage = self.cpu_load as u32;
        let terminate_flag = Arc::new(AtomicBool::new(false));

        let handles: Vec<_> = (0..num_cores)
            .map(|_| {
                let terminate_flag = terminate_flag.clone();
                thread::spawn(move || {
                    worker(percentage, terminate_flag);
                })
            })
            .collect();

        while node
            .wait(Duration::from_millis(self.interval_in_ms))
            .is_ok()
        {
            if self.event_triggering {
                for (i, sample) in self.samples.iter().enumerate() {
                    if !sample && self.listeners[i].blocking_wait().is_err() {
                        return;
                    }
                    while self.listeners[i].try_wait().unwrap() != 0 {}
                }
            }

            for (i, subscriber) in self.subscribers.iter().enumerate() {
                while let Some(l) = subscriber.receive(Box::new(|_| {})).unwrap() {
                    latency.set(l);
                    self.samples[i] = true;
                }
            }

            let mut has_all_samples = true;
            for sample in &self.samples {
                if !sample {
                    has_all_samples = false;
                    break;
                }
            }

            if has_all_samples {
                println!("latency: {}", latency);
                latency.reset();
                for (i, publisher) in self.publishers.iter().enumerate() {
                    publisher
                        .send(PAYLOAD_SIZE, Box::new(produce_data))
                        .unwrap();
                    if self.event_triggering {
                        self.notifiers[i].notify(0).unwrap();
                    }
                }

                for sample in &mut self.samples {
                    *sample = false;
                }
            }
        }

        // wrap up and terminate threads
        terminate_flag.store(true, Ordering::Relaxed);
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

type CommType = CommIceoryx2;

fn main() {
    let args = CliArgs::parse();

    match args.node {
        NodeType::Actor => {
            let mut actor = Actor::new::<CommType>(
                args.node_name,
                args.services,
                args.port_names,
                args.port_names_event,
                args.interval_in_ms,
                args.cpu_load,
                args.event_triggering,
            );
            actor.main_loop()
        }
        NodeType::Processing {
            out_services,
            out_ports,
            out_ports_event,
        } => {
            let mut processor = Processor::new::<CommType>(
                args.node_name,
                args.services,
                out_services,
                args.port_names,
                out_ports,
                args.port_names_event,
                out_ports_event,
                args.interval_in_ms,
                args.cpu_load,
                args.event_triggering,
            );
            processor.main_loop()
        }
        NodeType::Sensor => {
            let sensor = Sensor::new::<CommType>(
                args.node_name,
                args.services,
                args.port_names,
                args.port_names_event,
                args.interval_in_ms,
                args.cpu_load,
                args.event_triggering,
            );
            sensor.main_loop()
        }
    }
}
