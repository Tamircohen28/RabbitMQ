use std::net::{IpAddr};
#[allow(unused_imports)]
use log::{info, warn, error};
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use clap::{Arg, App};

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
    println!("Waiting for messages from {}...", recv_stream.peer_addr().unwrap());
    
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

    // clone strems before sending to threads
    let mut tcp_stream_1 = streams.pop().unwrap();
    let mut tcp_stream_1_copy = tcp_stream_1.try_clone().unwrap();
    let mut tcp_stream_2 = streams.pop().unwrap();
    let mut tcp_stream_2_copy = tcp_stream_2.try_clone().unwrap();

    let connection_a = thread::spawn(move|| {  
        handle_client(&mut tcp_stream_1, &mut tcp_stream_2); 
    });

    let connection_b = thread::spawn(move|| {  
        handle_client(&mut tcp_stream_2_copy, &mut tcp_stream_1_copy); 
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
