use std::net::{IpAddr};
use amiquip::{Connection, Exchange, Publish};
use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use log::{info, warn, error};
use std::thread;
use std::sync::{Arc};
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use clap::{Arg, App};
use env_logger::Env;

// consts
const MAX_MSG_SIZE: usize = 128;

static TCP_PORT_ARG : &str = "tcp_port";
static TCP_IP_ARG : &str = "tcp_ip";
static RAB_URL_ARG : &str = "rabbit_url";
static RAB_QUEUEU_SRC_ARG : &str = "rabbit_queue_src";
static RAB_QUEUEU_DST_ARG : &str = "rabbit_queue_dst";


struct Address {
    ip: IpAddr,
    port: u32 
}

struct RabbitAddress {
    url: String
}

struct RabbitConfig {
    address: RabbitAddress,
    src_queue: String,
    dst_queue: String
}

struct Config {
    rabbit: RabbitConfig,
    tcp: Address,
}

impl fmt::Display for Config {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config: \n")?;
        write!(f, "     Rabbit: \n")?;
        write!(f, "         url: {}\n", self.rabbit.address.url)?;
        write!(f, "         src queue: {}\n", self.rabbit.src_queue)?;
        write!(f, "         dat queue: {}\n", self.rabbit.dst_queue)?;
        write!(f, "     TCP: \n")?;
        write!(f, "         ip: {}\n", self.tcp.ip)?;
        write!(f, "         port: {}\n", self.tcp.port)
    }
}

fn parse_args() -> Option<Config> {
    let matches = App::new("Rabbit MQ to TCP Clint")
    .version("0.1.0")
    .author("Tamir Cohen <tamirc@mtazov.idf>")
    .about("Rustraining exresice")
    .arg(Arg::with_name("tcp_ip")
             .short("i")
             .long("tcp_ip")
             .takes_value(true)
             .help("tcp server address"))
    .arg(Arg::with_name(TCP_PORT_ARG)
             .short("p")
             .long(TCP_PORT_ARG)
             .takes_value(true)
             .help("A tcp server port acsess"))
    .arg(Arg::with_name(RAB_URL_ARG)
             .short("s")
             .long(RAB_URL_ARG)
             .takes_value(true)
             .help("A rabbit mq server url"))
    .arg(Arg::with_name(RAB_QUEUEU_SRC_ARG)
             .long(RAB_QUEUEU_SRC_ARG)
             .takes_value(true)
             .help("A queueu to receive messages from"))
    .arg(Arg::with_name(RAB_QUEUEU_DST_ARG)
             .long(RAB_QUEUEU_DST_ARG)
             .takes_value(true)
             .help("A queueu to send messages to"))
    .get_matches();

    let tcp_ip = matches.value_of(TCP_IP_ARG);
    let unwraped_ip : IpAddr;

    // check validity of ip and unwrap
    match tcp_ip {
        Some(ip) => { 
            if let Ok(valid_ip) = ip.parse::<IpAddr>() {
                unwraped_ip = valid_ip;
            }
            else
            {
                error!("value of argument <{}> cannot be parsed", TCP_IP_ARG);
                return None;
            }
        },
        None => {
            error!("argument <{}> is missing", TCP_IP_ARG);
            return None;
        }
    };

    let tcp_port = matches.value_of(TCP_PORT_ARG);
    let unwraped_port : u32;

    // check validity of ip and unwrap
    match tcp_port {
        Some(port) => { 
            if let Ok(valid_port) = port.parse::<u32>() {
                unwraped_port = valid_port;
            }
            else
            {
                error!("value of argument <{}> cannot be parsed", TCP_PORT_ARG);
                return None;
            }
        },
        None => {
            error!("argument <{}> is missing", TCP_PORT_ARG);
            return None;
        }
    };

    let rabbit_url = matches.value_of(RAB_URL_ARG);
    let unwraped_rabbit_url : String;

    // check validity of url and unwrap
    match rabbit_url {
        Some(url) => { 
            if let Ok(valid_url) = url.parse::<String>() {
                unwraped_rabbit_url = valid_url;
            }
            else
            {
                error!("value of argument <{}> cannot be parsed", RAB_URL_ARG);
                return None;
            }
        },
        None => {
            error!("argument <{}> is missing", RAB_URL_ARG);
            return None;
        }
    };

    let rabbit_queue_src = matches.value_of(RAB_QUEUEU_SRC_ARG);
    let unwraped_rabbit_queue_src : String;
    // check validity of queue src and unwrap
    match rabbit_queue_src {
        Some(queue) => { 
            if let Ok(valid_queue) = queue.parse::<String>() {
                unwraped_rabbit_queue_src = valid_queue;
            }
            else
            {
                error!("value of argument <{}> cannot be parsed", RAB_QUEUEU_SRC_ARG);
                return None;
            }
        },
        None => {
            error!("argument <{}> is missing", RAB_QUEUEU_SRC_ARG);
            return None;
        }
    };

    let rabbit_queue_dst = matches.value_of(RAB_QUEUEU_DST_ARG);
    let unwraped_rabbit_queue_dst : String;
    // check validity of queue dst and unwrap
    match rabbit_queue_dst {
        Some(queue) => { 
            if let Ok(valid_queue) = queue.parse::<String>() {
                unwraped_rabbit_queue_dst = valid_queue;
            }
            else
            {
                error!("value of argument <{}> cannot be parsed", RAB_QUEUEU_DST_ARG);
                return None;
            }
        },
        None => {
            error!("argument <{}> is missing", RAB_QUEUEU_DST_ARG);
            return None;
        }
    };

    let tcp_addr = Address {
        ip: unwraped_ip,
        port: unwraped_port,
    };

    let rabbit_addr = RabbitAddress {
        url: unwraped_rabbit_url
    };

    let rabbit_config = RabbitConfig {
        address: rabbit_addr,
        src_queue: unwraped_rabbit_queue_src,
        dst_queue: unwraped_rabbit_queue_dst
    };

    Some(Config { 
        rabbit : rabbit_config, 
        tcp : tcp_addr,
    })
}

/// run the tcp server
fn main() -> Result<(), ()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("client")).init();

    if let None = parse_args() {
        return Err(());
    }
    
    let config = Arc::new(parse_args().unwrap()); // here we can unwrap because we have Some(_)
    info!("\n{}", config);
    info!("Starting up Client...");
    let config_sender = config.clone();
    let config_recevier = config.clone();

    let tcp_url = format!("{}:{}", config.tcp.ip, config.tcp.port);

    // try connecting until sucsess
    let mut stream = TcpStream::connect(tcp_url.clone());

    while let Err(_) =  stream {
        stream = TcpStream::connect(tcp_url.clone());
    }
    let mut stream : TcpStream = stream.unwrap(); // here we can unwrap because we have Some(_)
    info!("TCP connection succsesful");

    // clone stream to splte between threads
    let mut stream_clone = match stream.try_clone() {
        Ok(clone) => clone,
        Err(e) => {
            error!("Stream clone had falied {:?}", e);
            return Err(());
        }
    };
    
    let rabbit_sender : std::thread::JoinHandle<std::result::Result<(), ()>> = thread::spawn(move || {
        // Open connection.
        let mut connection = match Connection::insecure_open(&config_sender.rabbit.address.url) {
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
        
        info!("Waiting for messages from TCP server...");
        // receive message from tcp and send to rabbit mq
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    warn!("Received empty message");
                    continue;
                }
                Ok(bytes_recv) => {
                    if let Err(_) = exchange.publish(Publish::new(&buffer[0..bytes_recv], config_sender.rabbit.dst_queue.clone())) {
                        warn!(" exchange publish had falied");
                    }
                    else
                    {
                        info!("Recv from TCP forwarding to queue '{}'", config_sender.rabbit.dst_queue);    
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
    });

    let rabbit_receiver = thread::spawn(move || {
        // Open connection.
        let mut connection = match Connection::insecure_open(&config_recevier.rabbit.address.url) {
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
        let queue = match channel.queue_declare(config_recevier.rabbit.src_queue.clone(), QueueDeclareOptions::default()) {
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
        info!("Waiting for messages from queue '{}'", config_recevier.rabbit.src_queue);

        for (i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    
                    info!("Received [{}], sending to TCP", i);

                    // send to tcp
                    if let Err(e) = stream_clone.write(&delivery.body) {
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
    });


    match rabbit_sender.join() {
        Ok(result) => {
            if let Err(_) = result {
                error!("error occured during rabbit_sender thread");
                return Err(());
            }
        },
        Err(_) => {
            error!("thread join had falied");
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
        Err(_) => {
            error!("thread join had falied");
            return Err(());
        }
    };

    Ok(())
}
