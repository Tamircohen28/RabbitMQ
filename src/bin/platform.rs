use std::process::Command;
use std::thread;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use log::{info};
use std::sync::Arc;
use env_logger::Env;

#[derive(Serialize, Deserialize, Debug)]
struct NetworkConfig {
    tcp_addr: String,
    rabbit_addr: String
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientConfig {
    src_queue: String,
    dst_queue: String
}

#[derive(Serialize, Deserialize, Debug)]
struct BinConfig {
    tcp_bin: String,
    rabbit_to_tcp_bin: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    client_1: ClientConfig,
    client_2: ClientConfig,
    network: NetworkConfig,
    binary: BinConfig
}

/// this run the platform proccess which then starts 2 rabbit to tcp client with
/// tcp server between them. 
/// 
/// QUEUE <-> CLIENT <-> TCP <-> CLIENT <-> QUEUE
/// 
/// this is the network after startup. 
/// panic! can only occure during init process.
fn main() {
    // set up logger
    env_logger::Builder::from_env(Env::default().default_filter_or("platform")).init();

    // parser configuration from json file
    let mut file = File::open(r"C:\Users\tamir\Desktop\RabbitMQ\configuration.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let config : Arc<Config> = Arc::new(serde_json::from_str(&data).unwrap());
    let config2 = config.clone();
    let config3 = config.clone();
    info!("Configuration loaded succesfuly");

    let tcp_procces = thread::spawn(move || {
        Command::new(&config.binary.tcp_bin)
        .args(&["-t", &config.network.tcp_addr])
        .status()
    });

    let client_procces = thread::spawn(move || {
        Command::new(&config2.binary.rabbit_to_tcp_bin)
        .args(&["-t", &config2.network.tcp_addr, 
                "-r", &config2.network.rabbit_addr,
                "-s", &config2.client_1.src_queue,
                "-d", &config2.client_1.dst_queue])
        .status()
    });

    let oppesite_client_procces = thread::spawn(move || {
        Command::new(&config3.binary.rabbit_to_tcp_bin)
        .args(&["-t", &config3.network.tcp_addr, 
                "-r", &config3.network.rabbit_addr,
                "-s", &config3.client_2.src_queue,
                "-d", &config3.client_2.dst_queue])
        .status()
    });
    
    tcp_procces.join().unwrap().unwrap();
    client_procces.join().unwrap().unwrap();
    oppesite_client_procces.join().unwrap().unwrap();
}
