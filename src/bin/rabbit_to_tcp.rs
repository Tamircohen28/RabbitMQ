use amiquip::{Connection, Exchange, Publish};
use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use log::{trace, info, warn, error};
use std::thread;
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

struct ConfigRabbitSender {
    rabbit_addr: String,
    dst_queue: String,
}

struct ConfigRabbitReceiver {
    rabbit_addr: String,
    src_queue: String,
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
fn tcp_to_rabbit (config : ConfigRabbitSender, mut recv_stream : TcpStream) -> Result<(), ()> {
    // Open connection.
    let mut connection = match Connection::insecure_open(&config.rabbit_addr) {
        Ok(connection) => connection,
        Err(e) => {
            error!("Connection opening had falied {:?}", e);
            return Err(());
        }
    };

    // Open a channel - None says let the library choose the channel ID.
    let channel = match connection.open_channel(None) {
        Ok(channel) => channel,
        Err(e) => {
            error!("Channel opening had falied {:?}", e);
            return Err(());
        }
    };

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

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
                if let Err(_) = exchange.publish(Publish::new(&buffer[0..bytes_recv], config.dst_queue.clone())) {
                    warn!("Exchange publish had falied");
                }
                else
                {
                    trace!("Recv from TCP forwarding to queue '{}'", config.dst_queue);    
                }
            },
            Err(e) => {
                error!("Could not read from stream {:?}", e);
                match connection.close() {
                    Err(e) => error!("Could not close stream {:?}", e),
                    Ok(_) => ()
                };
                return Err(());
            }
        };
    }
}

/// hendle communication from rabbit server to tcp
fn rabbit_to_tcp (config : ConfigRabbitReceiver, mut send_stream : TcpStream) -> Result<(), ()> {
    // Open connection.
    let mut connection = match Connection::insecure_open(&config.rabbit_addr) {
        Ok(connection) => connection,
        Err(e) => {
            error!("Connection opening had falied {:?}", e);
            return Err(());
        }
    };

    // Open a channel - None says let the library choose the channel ID.
    let channel = match connection.open_channel(None) {
        Ok(channel) => channel,
        Err(e) => {
            error!("Channel opening had falied {:?}", e);
            return Err(());
        }
    };

    // Declare the queue we receive the messages from
    let queue = match channel.queue_declare(config.src_queue.clone(), QueueDeclareOptions::default()) {
        Ok(queue) => queue,
        Err(e) => {
            error!("queue declare had falied {:?}", e);
            return Err(());
        }
    };

    // Start a consumer.
    let consumer = match queue.consume(ConsumerOptions::default()) {
        Ok(consumer) => consumer,
        Err(e) => {
            error!("consumer start had falied {:?}", e);
            return Err(());
        }
    };
    
    trace!("Waiting for messages from queue '{}'", config.src_queue);
    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                
                trace!("Received [{}], sending to TCP", i);
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
                break;
            }
        }
    }

    if let Err(e) = connection.close() {
        error!("connection close had falied {:?}", e);
        Err(())
    }
    else
    {
        Ok(())
    }
}

/// run the tcp server
fn main() -> Result<(), ()> {
    // set up logger
    env_logger::Builder::from_env(Env::default().default_filter_or("rabbit_to_tcp")).init();
    let config = Config::new();

    info!("\n{:#?}", config);
    info!("Starting up Rabbit to tcp Client...");

    // connecting to tcp server
    let send_stream = TcpStream::connect(config.tcp_addr).unwrap();

    // clone stream to splte between threads
    let recv_stream = send_stream.try_clone().unwrap();
    info!("TCP connection succsesful");
    
    // devide configuration into threads
    let sender_config = ConfigRabbitSender {
        rabbit_addr : config.rabbit_addr.clone(),
        dst_queue : config.dst_queue
    };

    let receiver_config = ConfigRabbitReceiver {
        rabbit_addr : config.rabbit_addr.clone(),
        src_queue : config.src_queue
    };

    // sender  threads
    let rabbit_sender = thread::spawn(move || {
        tcp_to_rabbit(sender_config, send_stream)
    });

    // receiver threads
    let rabbit_receiver = thread::spawn(move || {
        rabbit_to_tcp(receiver_config, recv_stream)
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