use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

struct ConnectionListener {
    tcp_listener: TcpListener,
    unattached_stream_sender: Sender<TcpStream>,
}

struct StreamListener {
    unassigned_streams: Vec<TcpStream>,
    unattached_stream_receiver: Receiver<TcpStream>,
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
        let mut stream_listener = StreamListener{
            unassigned_streams: vec![],
            unattached_stream_receiver,
        };
        println!("Stream listener is listening on connected streams...");
        stream_listener.run();
    });

    #[allow(clippy::empty_loop)]
    loop {}
}

impl StreamListener {
    fn run(&mut self) {
        loop {
            self.receive_unattached_streams();

            self.listen_unassigned_streams();

//            self.list_attached_streams();
        }
    }

    fn listen_unassigned_streams(&mut self) {
        for tcp_stream in &mut self.unassigned_streams {
            let mut message_bytes = [0; 512];
            tcp_stream.set_read_timeout(Some(Duration::from_millis(10))).unwrap();
            let num_bytes_read = match tcp_stream.read(&mut message_bytes) {
                Ok(n) => n,
                Err(_) => 0,
            };
            if num_bytes_read > 0 {
                let mut in_message = String::from(std::str::from_utf8(&message_bytes).unwrap());
                in_message.truncate(num_bytes_read);
                println!("Received: {}", in_message);
            }
        }
    }

    fn receive_unattached_streams(&mut self) {
        let receive_result = self.unattached_stream_receiver.try_recv();
        match receive_result {
            Ok(tcp_stream) => { self.received_unattached_stream(tcp_stream) },
            Err(try_receieve_error) => {
                match try_receieve_error {
                    TryRecvError::Empty => {},
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
