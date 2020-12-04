use amiquip::{Connection, Exchange, Publish};
use amiquip::{Consumer, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use log::{trace, info, warn, error};
use std::thread;
use std::sync::Arc;
use std::io::prelude::*;
use std::net::TcpStream;
use env_logger::Env;
#[macro_use]
extern crate clap;
use clap::{App};

// consts
const MAX_MSG_SIZE: usize = 128;

#[derive(Debug)]
struct Config {
    rabbit_addr: String,
    tcp_addr: String,
    src_queue: String,
    dst_queue: String
}

impl Config {
    /// create new config struct by parsering the arguments
    fn new() -> Self {
        let yaml = load_yaml!(r"C:\Users\tamir\Desktop\RabbitMQ\yml\client_cli.yaml");
        let matches = App::from_yaml(yaml).get_matches(); 
        
        Config {
            tcp_addr : String::from(matches.value_of("tcp_addr").unwrap()),
            rabbit_addr : String::from(matches.value_of("rabbit_addr").unwrap()),
            src_queue : String::from(matches.value_of("src_queue").unwrap()),
            dst_queue : String::from(matches.value_of("dst_queue").unwrap()),
        }
    }
}

/// handle communication from tcp to rabbit server
fn tcp_to_rabbit (exchange : Exchange, mut recv_stream : TcpStream, dst_queue : String) -> Result<(), ()> {
    // iter over all messages received from tcp
    let mut buffer = [0 as u8; MAX_MSG_SIZE];
    
    trace!("Waiting for messages from TCP server...");
    // receive message from tcp and send to rabbit mq
    loop {
        match recv_stream.read(&mut buffer) {
            Ok(0) => {
                warn!("Received empty message");
                continue;
            }
            Ok(bytes_recv) => {
                if let Err(_) = exchange.publish(Publish::new(&buffer[0..bytes_recv], dst_queue.clone())) {
                    warn!("Exchange publish had falied");
                }
                else
                {
                    trace!("Recv from TCP forwarding to queue '{}'", dst_queue);    
                }
            },
            Err(e) => {
                error!("Could not read from stream {:?}", e);
                return Err(());
            }
        };
    }
}

/// hendle communication from rabbit server to tcp
fn rabbit_to_tcp (consumer : Consumer, mut send_stream : TcpStream) -> Result<(), ()> {

    trace!("Waiting for messages from Rabbit server...");
    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                // send to tcp
                if let Err(e) = send_stream.write(&delivery.body) {
                    error!("stream write had falied {:?}", e);
                    return Err(());
                }

                // acknowlage rabbit server
                if let Err(e) = consumer.ack(delivery) {
                    error!("consumer acknolage had falied {:?}", e);
                    return Err(());
                }
            }
            other => {
                warn!("Consumer ended: {:?}", other);
                return Err(());
            }
        }
    }
    Ok(())
}

/// run the tcp server
fn main() -> Result<(), ()> {
    // set up logger
    env_logger::Builder::from_env(Env::default().default_filter_or("rabbit_to_tcp")).init();
    let config = Arc::new(Config::new());
    let config_copy = config.clone();
    info!("Starting up Rabbit to tcp Client...");

    // connecting to tcp server
    let send_stream = TcpStream::connect(config.tcp_addr.clone()).unwrap();

    // clone stream to splte between threads
    let recv_stream = send_stream.try_clone().unwrap();
    info!("TCP connection succsesful");

    // sender thread
    let rabbit_sender = thread::spawn(move || {
        // Open connection.
        let mut connection = Connection::insecure_open(&config.rabbit_addr).unwrap();
        // Open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None).unwrap();
        // Get a handle to the direct exchange on our channel.
        let exchange = Exchange::direct(&channel);
        info!("Exchange creation succsesful");

        tcp_to_rabbit(exchange, send_stream, config.dst_queue.clone())
    });

    // receiver threads
    let rabbit_receiver = thread::spawn(move || {
        // Open connection.
        let mut connection = Connection::insecure_open(&config_copy.rabbit_addr).unwrap();
        // Open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None).unwrap();
        // Declare the queue we receive the messages from
        let queue = channel.queue_declare(config_copy.src_queue.clone(), QueueDeclareOptions::default()).unwrap();
        // Start a consumer.
        let consumer = queue.consume(ConsumerOptions::default()).unwrap();

        rabbit_to_tcp(consumer, recv_stream)
    });

    match rabbit_sender.join() {
        Ok(result) => {
            if let Err(_) = result {
                error!("error occured during rabbit_sender thread");
                return Err(());
            }
        },
        Err(e) => {
            error!("thread join had falied {:?}", e);
            return Err(());
        }
    };

    match rabbit_receiver.join() {
        Ok(result) => {
            if let Err(_) = result {
                error!("error occured during rabbit_receiver thread");
                return Err(());
            }
        },
        Err(e) => {
            error!("thread join had falied {:?}", e);
            return Err(());
        }
    };

    Ok(())
}