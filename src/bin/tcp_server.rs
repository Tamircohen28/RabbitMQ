use log::{trace, info, warn, error};
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use env_logger::Env;
#[macro_use]
extern crate clap;
use clap::{App};

//consts
const MAX_MSG_SIZE : usize = 128;
const MAX_CONNECTIONS : usize = 2;

#[derive(Debug)]
struct Config {
    tcp_addr: String,
}

impl Config {
    /// create new config struct by parsering the arguments
    fn new() -> Self {
        let yaml = load_yaml!(r"C:\Users\tamir\Desktop\RabbitMQ\yml\tcp.yaml");
        let matches = App::from_yaml(yaml).get_matches(); 
        
        Config {
            tcp_addr : String::from(matches.value_of("tcp_addr").unwrap()),
        }
    }
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
            Ok(0) => warn!("Empty message from {}", recv_addr),
            Ok(bytes_recv) => {
                trace!("Received {} bytes from {}", bytes_recv, recv_addr);
                
                if let Err(e) = send_stream.write(&buffer[..bytes_recv]) {
                    error!("failed writing to stream '{}'", e);
                    return Err(());
                }
            },
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
    // set up
    env_logger::Builder::from_env(Env::default().default_filter_or("tcp_server")).init();
    let config = Config::new();
    info!("Starting up TCP Server...");
    
    // binding to port
    let listener = TcpListener::bind(config.tcp_addr).unwrap();
    info!("bind succesful");
    let mut streams : Vec<TcpStream> = Vec::new();
    trace!("Server listening for connections...");

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
    let mut tcp_stream_1 = streams.pop().unwrap();
    let mut tcp_stream_1_copy = tcp_stream_1.try_clone().unwrap();
    let mut tcp_stream_2 = streams.pop().unwrap();
    let mut tcp_stream_2_copy = tcp_stream_2.try_clone().unwrap();
    
    // hendle each connection on new thread
    let connection_a = thread::spawn(move|| {  
        handle_client(&mut tcp_stream_1, &mut tcp_stream_2)
    });

    let connection_b = thread::spawn(move|| {  
        handle_client(&mut tcp_stream_2_copy, &mut tcp_stream_1_copy)
    });

    // wait for threads outcome
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
