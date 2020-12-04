use std::process::Command;
use std::thread;
use serde::{Deserialize, Serialize};

// this reads file at compile times
const CONFIG_JSON: &str = include_str!(r"C:\Users\tamir\Desktop\RabbitMQ\configuration.json");

#[derive(Serialize, Deserialize, Debug)]
struct Porccess {
    binary_path: String,
    args: String
}

/// this run the platform proccess which then starts 2 rabbit to tcp client with
/// tcp server between them. 
/// 
/// QUEUE <-> CLIENT <-> TCP <-> CLIENT <-> QUEUE
/// 
/// this is the network after startup. 
/// panic! can only occure during init process.
fn main() {
    let config : Vec<Porccess>= serde_json::from_str(&CONFIG_JSON).unwrap();
    let mut handler_vec = vec!();

    // run all proccess
    for process_args in config {
        let handler = thread::spawn(move || {
            Command::new(process_args.binary_path).args(process_args.args.split(" ").collect::<Vec<&str>>()).status()
        });

        handler_vec.push(handler);
    }

    // wait for them to finish
    for handler in handler_vec {
        handler.join().unwrap().unwrap();
    }
}
