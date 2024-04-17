mod client;

use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32; // Buffer size of messages

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");

    // nonblocking allows the server to constantly check for messages
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    let mut clients = Vec::new();

    // Instantiate a channel which will receive strings
    let (tx, rx) = mpsc::channel::<String>();

    // Infinite loop that checks for incoming connections
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            // Read clients name as first msg
            let mut name_buff = vec![0; MSG_SIZE];
            let mut client_name = if let Ok(_) = socket.read_exact(&mut name_buff) {
                String::from_utf8(name_buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>()).expect("Invalid name")
            } else { "Unknowm".to_string() };

            println!("{} connected as {}", addr, client_name);
            clients.push((socket.try_clone().expect("Failed to clone client"), client_name.clone()));

            // Spawn thread to handle each client
            thread::spawn(move || {
                let mut buff = vec![0; MSG_SIZE];
                loop {
                    // Buffer to store msg data from client
                    let mut buff = vec![0; MSG_SIZE];
                    match socket.read_exact(&mut buff) {
                        Ok(_) => {
                            // Collect all char that is not whitespaces and into a string
                            let msg = String::from_utf8(buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>()).expect("Invalid UFT-8 msg");
                            let formatted_msg = format!("{}: {}", client_name, msg);

                            println!("{:?}", formatted_msg);
                            tx.send(formatted_msg).expect("Failed to send msg to transmitter");
                        },
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("Closing connection with: {}", addr);
                            break;
                        }
                    }
                    sleep();
                }
            });
        }

        // Checks for msg from receiver
        if let Ok(msg) = rx.try_recv() {
            // Iterates over each client in the vec to send recv msg
            clients = clients.into_iter().filter_map(|(mut client, name)| {
                // msg into bytes, and fits into the size of MSG_SIZE
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                let full_message = format!("{}: {}", name, msg);

                // Attempts to send buffered msg to client
                // '.map(|_| client, name)' transforms the successful result into the client itself
                client.write_all(&full_message.into_bytes())
                    .map(|_| (client, name))
                    .ok()
            }).collect::<Vec<_>>();
        }
        sleep();
    }
}
