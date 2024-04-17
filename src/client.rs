use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::{io, thread};
use std::time::Duration;
use io::stdin;
const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32; // Buffer size of messages

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Failed to connect");
    client.set_nonblocking(true).expect("Failed to initialize non-blocking");

    // Instantiate a channel which will receive strings
    let (tx, rx) = mpsc::channel::<String>();

    println!("Enter your name:");
    let mut name = String::new();
    stdin().read_line(&mut name).expect("Reading from stdin failed");

    // Send name to the server
    let mut name_buff = name.clone().into_bytes();
    name_buff.resize(MSG_SIZE, 0);
    client.write_all(&name_buff).expect("Writing a name to socket failed");

    // Spawn thread for handling server communication
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        // Read MSG_SIZE bytes from server into buff
        match client.read_exact(&mut buff) {
            Ok(_) => {
                // Convert buffer into a string
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Message received: {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was disconnected");
                break;
            }
        }

        // Try to receive msg from channel to send to the server
        match rx.try_recv() {
            Ok(msg) => { // Send to server
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                println!("Message send {:?}", msg);
            },
            // Ignore empty err
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
        // Sleep for less CPU usage...
        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {break}
    }
    println!("Thanks for using my chat");
}