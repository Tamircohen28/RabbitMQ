use std::net::{IpAddr};
use amiquip::{Connection, Exchange, Publish, Result};
use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use std::thread;

#[allow(dead_code)]
struct Address {
    ip: IpAddr,
    port: u32
}

#[allow(dead_code)]
struct Config {
    src: Address,
    dst: Address
}

#[allow(dead_code)]
fn parse_args() -> Config {
    let addr = Address {
        ip: "127.0.0.1".parse().unwrap(),
        port: 9999
    };

    let addr2 = Address {
        ip: "127.0.0.1".parse().unwrap(),
        port: 9999
    };

    Config { 
        src : addr, 
        dst : addr2,
     }
}

fn send_msg() -> Result<()> {
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    // Publish a message to the "hello" queue.
    exchange.publish(Publish::new("hello there".as_bytes(), "hello"))?;

    connection.close()
}

fn recv_msg() -> Result<()> {
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Declare the "hello" queue.
    let queue = channel.queue_declare("hello", QueueDeclareOptions::default())?;

    // Start a consumer.
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages. Press Ctrl-C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                println!("({:>3}) Received [{}]", i, body);
                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let config = parse_args();
        assert_eq!(config.src.port, 9999);
    }

    #[test]
    fn run() {
        thread::spawn(|| {
            println!("Sending messgae...");
            send_msg().unwrap();
        });

        println!("Receving message...");
        recv_msg().unwrap();
    }
}