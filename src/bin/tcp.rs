use std::net::{IpAddr};
use log::{info, warn, error};
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use clap::{Arg, App};
use std::fmt;
use env_logger::Env;

//consts
const MAX_MSG_SIZE : usize = 128;
const MAX_CONNECTIONS : usize = 2;

static TCP_PORT_ARG : &str = "tcp_port";
static TCP_IP_ARG : &str = "tcp_ip";

struct TCPAddress {
    ip: IpAddr,
    port: u32 
}

struct Config {
    tcp: TCPAddress
}

impl fmt::Display for Config {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config: \n")?;
        write!(f, "         ip: {}\n", self.tcp.ip)?;
        write!(f, "         port: {}\n", self.tcp.port)
    }
}

/// hendle the argument parsing of the progrem
/// return configuration struct option
fn parse_args() -> Option<Config> {
    let matches = App::new("TCP process between clients")
    .version("0.1.0")
    .author("Tamir Cohen <tamirc@mtazov.idf>")
    .about("TCP process")
    .arg(Arg::with_name(TCP_IP_ARG)
             .short("i")
             .long(TCP_IP_ARG)
             .takes_value(true)
             .help("ip to bind to"))
    .arg(Arg::with_name(TCP_PORT_ARG)
             .short("p")
             .long(TCP_PORT_ARG)
             .takes_value(true)
             .help("port to bind to"))
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

    Some(Config {
        tcp : 
            TCPAddress {
                ip: unwraped_ip,
                port: unwraped_port
            }
    })
}

/// hendle client connectig to server, forward message from one stream to the other
fn handle_client(recv_stream: &mut TcpStream, send_stream: &mut TcpStream) -> Result<(), ()> {
    let mut buffer = [0 as u8; MAX_MSG_SIZE];

    let recv_addr = match recv_stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!("falied reading socket address '{}'", e);
            return Err(());
        }
    };
    
    info!("Waiting for messages from {}...", recv_addr);
    loop {
        match recv_stream.read(&mut buffer) {
            // empty message
            Ok(0) => {
                warn!("Empty message from {}", recv_addr);
            },
            // valid message
            Ok(bytes_recv) => {
                info!("Received {} bytes from {}", bytes_recv, recv_addr);
                
                if let Err(e) = send_stream.write(&buffer[0..bytes_recv]) {
                    error!("failed writing to stream '{}'", e);
                    return Err(());
                }
            },
            // connection error
            Err(e) => {
                error!("An error occurred, terminating connection with {} : '{}'", recv_addr, e);
                if let Err(e) = recv_stream.shutdown(Shutdown::Both) {
                    error!("falied closing stream '{}'", e);
                }
            }
        }
    }
}

/// run the tcp server
fn main() -> Result<(), ()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("tcp")).init();

    if let None = parse_args() {
        return Err(());
    }

    let config = parse_args().unwrap();

    info!("\n{}", config);
    info!("Starting up TCP Server...");
    
    // try connecting until sucsess
    let mut listener = TcpListener::bind(format!("{}:{}", config.tcp.ip, config.tcp.port));
    while let Err(_) =  listener {
        listener = TcpListener::bind(format!("{}:{}", config.tcp.ip, config.tcp.port)); 
    }

    let listener : TcpListener = listener.unwrap(); // here listener is Some(_) so we can unwrap
    info!("bind succesful");
    let mut streams : Vec<TcpStream> = Vec::new();
    info!("Server listening on port {}", config.tcp.port);

    // accept connections until reach max
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection: {}", stream.peer_addr().unwrap());
                streams.push(stream);
                
                if streams.len() >= MAX_CONNECTIONS {
                    info!("Received max conections possible [{}]", MAX_CONNECTIONS);
                    break;
                }
            }
            Err(e) => {
                /* connection failed */
                warn!("stream connection had falied '{}'", e);
            }
        }
    }    

    // clone strems before sending to threads
    let mut tcp_stream_1 = match streams.pop() {
        Some(stream) => stream,
        None => {
            error!("Stream pop had falied");
            return Err(());
        }
    };

    let mut tcp_stream_1_copy = match tcp_stream_1.try_clone() {
        Ok(clone) => clone,
        Err(_) => {
            error!("Stream try_clone falied");
            return Err(());
        }
    };

    let mut tcp_stream_2 = match streams.pop() {
        Some(stream) => stream,
        None => {
            error!("Stream pop had falied");
            return Err(());
        }
    };

    let mut tcp_stream_2_copy = match tcp_stream_2.try_clone() {
        Ok(clone) => clone,
        Err(_) => {
            error!("Stream try_clone falied");
            return Err(());
        }
    };

    let connection_a = thread::spawn(move|| {  
        handle_client(&mut tcp_stream_1, &mut tcp_stream_2)
    });

    let connection_b = thread::spawn(move|| {  
        handle_client(&mut tcp_stream_2_copy, &mut tcp_stream_1_copy)
    });

    match connection_a.join() {
        Ok(result) => {
            if let Err(_) = result {
                error!("error occured during connection thread");
                return Err(());
            }
        },
        Err(_) => {
            error!("thread join had falied");
            return Err(());
        }
    };

    match connection_b.join() {
        Ok(result) => {
            if let Err(_) = result {
                error!("error occured during connection thread");
                return Err(());
            }
        },
        Err(_) => {
            error!("thread join had falied");
            return Err(());
        }
    };
    
    // close the socket server
    drop(listener);
    Ok(())
}
