use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread;

struct ChatServer {
    tcp_listener: TcpListener,
    unattached_stream_sender: Sender<TcpStream>,
}


pub fn run(port: u16) {
    let (unattached_stream_sender, unattached_stream_receiver) = channel();

    thread::spawn(move || {
        let endpoint_str = format!("127.0.0.1:{0}", port);
        let tcp_listener = TcpListener::bind(endpoint_str.clone())
            .unwrap_or_else(|_| panic!("Failed to bind tcp listener to endpoint:\n {0}", endpoint_str));
        let chat_server = ChatServer {
            tcp_listener,
            unattached_stream_sender,
        };
        println!("Chat server is listening for new connections...");
        chat_server.run();
    });

    thread::spawn(move || {
       loop {
           let receive_result = unattached_stream_receiver.try_recv();
           match receive_result {
               Ok(_tcp_stream) => println!("Got something"),
               Err(try_receieve_error) => {
                   match try_receieve_error {
                       TryRecvError::Empty => {},
                       TryRecvError::Disconnected => println!("disconnected from channel"),
                   }
               }
           }
       }
    });

    #[allow(clippy::empty_loop)]
    loop {}
}

impl ChatServer {
    fn run(self) {
        loop {
            self.accept_connections();
        }
    }

    fn accept_connections(&self) {
        let (tcp_stream, socket_addr) = self.tcp_listener.accept().unwrap();
        println!("Accepted new connection {}", socket_addr.port());
        tcp_stream.set_nonblocking(true).expect("failed to set tcp stream to nonblocking");
        self.unattached_stream_sender.send(tcp_stream).expect("channel for unattached streams is broken");
    }
}
