use amiquip::{Connection, Exchange, Publish};
use log::{trace};
use rand::Rng;
use env_logger::Env;

// consts
const MAX_MSG_SIZE: usize = 128;
const NUM_OF_MSG: usize = 10;

fn main() {
    // set up logger
    env_logger::Builder::from_env(Env::default().default_filter_or("msg_generator")).init();

    // Open connection.
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None).unwrap();

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    // iter over all messages received from tcp
    let mut buffer = [0 as u8; MAX_MSG_SIZE];
    let mut rng = rand::thread_rng();

    for msg in 0..NUM_OF_MSG {
        trace!("Generating msg number {}", msg);

        for i in 0..buffer.len() {
            buffer[i] = rng.gen();
        }
        
        exchange.publish(Publish::new(&buffer, "starta_test_queue")).unwrap();
    }
}