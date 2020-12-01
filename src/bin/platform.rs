use std::process::Command;
use std::thread;

fn main() {
    let tcp_procces = thread::spawn(|| {
        Command::new("tcp.exe")
        .current_dir(r"C:\Users\tamir\Desktop\RabbitMQ\target\debug")
        .args(&["-i", "127.0.0.1", "-p", "9999"])
        .status()
    });

    let client_procces = thread::spawn(|| {
        Command::new("client.exe")
        .current_dir(r"C:\Users\tamir\Desktop\RabbitMQ\target\debug")
        .args(&["-i",
                "127.0.0.1",
                "-p",
                "9999",
                "-s",
                "amqp://guest:guest@localhost:5672",
                "--rabbit_queue_src",
                "starta_test_queue",
                "--rabbit_queue_dst",
                "enda_test_queue"])
        .status()
    });

    let oppesite_client_procces = thread::spawn(|| {
        Command::new("client.exe")
        .current_dir(r"C:\Users\tamir\Desktop\RabbitMQ\target\debug")
        .args(&["-i",
                "127.0.0.1",
                "-p",
                "9999",
                "-s",
                "amqp://guest:guest@localhost:5672",
                "--rabbit_queue_src",
                "startb_test_queue",
                "--rabbit_queue_dst",
                "endb_test_queue"])
        .status()
    });
    
    tcp_procces.join().unwrap().unwrap();
    client_procces.join().unwrap().unwrap();
    oppesite_client_procces.join().unwrap().unwrap();
}
