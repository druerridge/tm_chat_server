use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::commands::ConnectionCommand;
use serde_json::Error;
use std::collections::HashMap;

struct ConnectionListener {
    tcp_listener: TcpListener,
    unattached_stream_sender: Sender<TcpStream>,
}

struct StreamListener {
    unassigned_streams: Vec<TcpStream>,
    unattached_stream_receiver: Receiver<TcpStream>,
    tcp_streams_by_room_id: HashMap<RoomId, Vec<TcpStream>>,
}

pub fn run(port: u16) {
    let (unattached_stream_sender, unattached_stream_receiver) = channel();

    thread::spawn(move || {
        let endpoint_str = format!("127.0.0.1:{0}", port);
        let tcp_listener = TcpListener::bind(endpoint_str.clone())
            .unwrap_or_else(|_| panic!("Failed to bind tcp listener to endpoint:\n {0}", endpoint_str));
        let connection_listener = ConnectionListener {
            tcp_listener,
            unattached_stream_sender,
        };
        println!("Chat server is listening for new connections...");
        connection_listener.run();
    });

    thread::spawn(move || {
        let mut stream_listener = StreamListener {
            unassigned_streams: vec![],
            unattached_stream_receiver,
            tcp_streams_by_room_id: HashMap::new(),
        };
        println!("Stream listener is listening on connected streams...");
        stream_listener.run();
    });

    #[allow(clippy::empty_loop)]
        loop {}
}

type RoomId = String;

impl StreamListener {
    fn run(&mut self) {
        loop {
            self.receive_unattached_streams();

            self.listen_unassigned_streams();

//            self.list_attached_streams();
        }
    }

    fn listen_unassigned_streams(&mut self) {
        let mut i = self.unassigned_streams.len();
        while i > 0 {
            i -= 1;
            let tcp_stream = self.unassigned_streams.get_mut(i).expect("index out of bounds");
            let option_message = StreamListener::read(tcp_stream);
            if let Some(connection_command) = StreamListener::connection_command_from(option_message) {
                let tcp_stream = self.unassigned_streams.remove(i);
                self.add_to_room(connection_command, tcp_stream);
            }
        }
    }

    fn write(tcp_stream: &mut TcpStream, out_message: &str) {
        println!("Sending: {}", out_message);
        let _ = tcp_stream.write(out_message.as_bytes()).expect("error writing to tcp stream");
        tcp_stream.flush().expect("Error flushing tcpstream after write");
    }

    fn add_to_room(&mut self, connection_command: ConnectionCommand, tcp_stream: TcpStream) {
        match self.tcp_streams_by_room_id.get_mut(connection_command.room.as_str()) {
            Some(vec) => {
                vec.push(tcp_stream);
                for tcp_stream in vec {
                    StreamListener::write(tcp_stream, &format!("{0} joined the room", connection_command.name));
                }
            }
            None => {
                let vec = vec![tcp_stream];
                self.tcp_streams_by_room_id.insert(connection_command.room, vec);
            }
        }
    }

    fn connection_command_from(option_message: Option<String>) -> Option<ConnectionCommand> {
        if let Some(message) = option_message {
            let result: Result<ConnectionCommand, Error> = serde_json::from_str(message.as_str());
            match result {
                Ok(connection_command) => {
                    println!("Join {0}", connection_command.room);
                    return Some(connection_command);
                }
                Err(e) => {
                    eprintln!("error parsing connection string: {0}", e);
                    return None;
                }
            }
        }
        None
    }

    fn read(tcp_stream: &mut TcpStream) -> Option<String> {
        let mut message_bytes = [0; 512];
        tcp_stream.set_read_timeout(Some(Duration::from_millis(10))).unwrap();
        let num_bytes_read = match tcp_stream.read(&mut message_bytes) {
            Ok(n) => n,
            Err(_) => 0,
        };
        if num_bytes_read > 0 {
            let mut message = String::from(std::str::from_utf8(&message_bytes).unwrap());
            message.truncate(num_bytes_read);
            println!("Received: {}", message);
            return Some(message);
        }
        None
    }

    fn receive_unattached_streams(&mut self) {
        let receive_result = self.unattached_stream_receiver.try_recv();
        match receive_result {
            Ok(tcp_stream) => { self.received_unattached_stream(tcp_stream) }
            Err(try_receieve_error) => {
                match try_receieve_error {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => println!("disconnected from channel"),
                }
            }
        }
    }

    fn received_unattached_stream(&mut self, tcp_stream: TcpStream) {
        println!("Received new unattached stream");
        self.unassigned_streams.push(tcp_stream);
    }
}

impl ConnectionListener {
    fn run(self) {
        loop {
            self.accept_connections();
        }
    }

    fn accept_connections(&self) {
        let (tcp_stream, socket_addr) = self.tcp_listener.accept().unwrap();
        println!("Accepted new connection {}", socket_addr.port());
        self.unattached_stream_sender.send(tcp_stream).expect("channel for unattached streams is broken");
    }
}
