use std::net::{IpAddr};
#[allow(unused_imports)]
use log::{info, warn, error};
use std::thread;
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

struct TCPAddress {
    ip: IpAddr,
    port: u32 
}

struct Config {
    tcp: TCPAddress
}

fn parse_args() -> Config {
    let addr = TCPAddress {
        ip: "127.0.0.1".parse().unwrap(),
        port: 9999
    };

    Config {
        tcp : addr,
    }
}

const MAX_MSG_SIZE : usize = 128;
const MAX_CONNECTIONS : usize = 2;

fn handle_client(recv_stream: &mut TcpStream, send_stream: &mut TcpStream) {
    let mut data = [0 as u8; MAX_MSG_SIZE];
    
    while match recv_stream.read(&mut data) {
        Ok(size) => {
            println!("Received {} bytes from {}", size, recv_stream.peer_addr().unwrap());
            // echo everything!
            send_stream.write(&data[0..size]).unwrap();
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", recv_stream.peer_addr().unwrap());
            recv_stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    let config = parse_args();
    let listener = TcpListener::bind(format!("{}:{}", config.tcp.ip, config.tcp.port)).unwrap();
    let mut streams : Vec<TcpStream> = Vec::new();

    // accept connections until reach max
    println!("Server listening on port {}", config.tcp.port);

    for stream in listener.incoming() {
        
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                streams.push(stream);
                
                if streams.len() >= MAX_CONNECTIONS {
                    println!("Received max conections possible [{}]", MAX_CONNECTIONS);
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }    
    let streams = Arc::new(Mutex::new(streams));
    let streams_copy = streams.clone();

    let connection_a = thread::spawn(move|| {  
        let mut streams = streams.lock().unwrap(); 
        let (src, dst) = streams.split_at_mut(1);
        let src = &mut src[0];
        let dst = &mut dst[0]; 
        handle_client(src, dst); 
    });

    let connection_b = thread::spawn(move|| {   
        let mut streams = streams_copy.lock().unwrap();
        let (src, dst) = streams.split_at_mut(1);
        let src = &mut src[0];
        let dst = &mut dst[0]; 
        handle_client(dst, src); 
    });

    if let Err(e) = connection_a.join() {
        println!("connection_a: {:?}", e);
    }

    if let Err(e) = connection_b.join() {
        println!("connection_b: {:?}", e);
    }

    // close the socket server
    drop(listener);
}
