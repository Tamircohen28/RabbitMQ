use std::net::{IpAddr};
use amiquip::{Connection, Exchange, Publish};
use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
#[allow(unused_imports)]
use log::{info, warn, error};
use std::thread;
use std::sync::{Arc, Mutex};
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use clap::{Arg, App};

struct Address {
    ip: IpAddr,
    port: u32 
}

struct RabbitAddress {
    url: String
}

struct Config {
    rabbit: RabbitAddress,
    tcp: Address
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
    .arg(Arg::with_name("tcp_port")
             .short("p")
             .long("tcp_port")
             .takes_value(true)
             .help("A tcp server port acsess"))
    .arg(Arg::with_name("rabbit_server")
             .short("s")
             .long("rabbit_server")
             .takes_value(true)
             .help("A rabbit mq server url"))
    .get_matches();

    let tcp_ip = matches.value_of("tcp_ip");
    match tcp_ip {
        Some(ip) => { 
            ip.parse::<IpAddr>().expect("<tcp_ip> value is invalid!");
        },
        None => {
            println!("tcp_ip is missing!");
            return None;
        }
    };

    let tcp_port = matches.value_of("tcp_port");
    match tcp_port {
        Some(ip) => { 
            ip.parse::<u32>().expect("<tcp_port> value is invalid!");
        },
        None => {
            println!("tcp_port is missing!");
            return None;
        }
    };

    let rabbit_url = matches.value_of("rabbit_server");
    match rabbit_url {
        Some(url) => {
            url.parse::<String>().expect("<rabbit_server> value is invalid!");
        },
        None => {
            println!("rabbit_server is missing!");
            return None;
        }
    };

    let tcp_addr = Address {
        ip: tcp_ip.unwrap().parse().unwrap(),
        port: tcp_port.unwrap().parse().unwrap(),
    };

    let rabbit_addr = RabbitAddress {
        url: String::from(rabbit_url.unwrap())
    };

    Some(Config { 
        rabbit : rabbit_addr, 
        tcp : tcp_addr,
    })
}

impl fmt::Display for Config {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config: \n")?;
        write!(f, "     Rabbit: \n")?;
        write!(f, "         url: {}\n", self.rabbit.url)?;
        write!(f, "     TCP: \n")?;
        write!(f, "         ip: {}\n", self.tcp.ip)?;
        write!(f, "         port: {}\n", self.tcp.port)
    }
}

static SRC_QUEUE_NAME: &str = "src_test_queue";
static DST_QUEUE_NAME: &str = "dst_test_queue";
const MAX_MSG_SIZE: usize = 128;

fn main() {
    let config = Arc::new(parse_args().unwrap()); 
    println!("{}", config);
    let config_sender = config.clone();
    let config_recevier = config.clone();

    let stream = TcpStream::connect(format!("{}:{}", config.tcp.ip, config.tcp.port)).unwrap();
    let stream = Arc::new(Mutex::new(stream));
    let send_stream = stream.clone();
    let recv_stream = stream.clone();
    
    let rabbit_sender = thread::spawn(move || {
        // Open connection.
        let mut connection = Connection::insecure_open(&config_sender.rabbit.url)?;

        // Open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None)?;

        // Get a handle to the direct exchange on our channel.
        let exchange = Exchange::direct(&channel);

        // iter over all messages received from tcp
        let mut buffer = [0 as u8; MAX_MSG_SIZE];

        // receive message from tcp and send to rabbit mq
        loop {
            println!("checking for messages");
            match recv_stream.lock().unwrap().read(&mut buffer) {
                Ok(bytes_recv) if bytes_recv > 0 => {
                    exchange.publish(Publish::new(&buffer[0..bytes_recv], DST_QUEUE_NAME))?;
                    println!("receive data from tcp sending to rabbit mq {}", DST_QUEUE_NAME);
                },
                Ok(_) => continue,
                Err(e) => {
                    eprint!("{:?}",e);
                    break
                }
            };
            println!("checking for messages");
        }

        connection.close()
    });

    let rabbit_receiver = thread::spawn(move || {
        // Open connection.
        let mut connection = Connection::insecure_open(&config_recevier.rabbit.url)?;

        // Open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None)?;

        // Declare the queue we receive the messages from
        let queue = channel.queue_declare(SRC_QUEUE_NAME, QueueDeclareOptions::default())?;

        // Start a consumer.
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("Waiting for messages... from queue {}", SRC_QUEUE_NAME);

        for (i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    println!("({:>3}) Received [{}] sending to TCP", i, body);

                    // send to tcp
                    send_stream.lock().unwrap().write(&delivery.body).unwrap();

                    println!("done senfing!");
                    // acknowlage rabbit server
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }

        connection.close()
    });

    if let Err(e) = rabbit_sender.join() {
        println!("rabbit_sender: {:?}", e);
    }

    if let Err(e) = rabbit_receiver.join() {
        println!("rabbit_receiver: {:?}", e);
    }
}
