use std::net::TcpListener;
use std::thread;

struct ChatServer {
    tcp_listener: TcpListener,
}


pub fn run(port: u16) {
    thread::spawn(move || {
        let endpoint_str = format!("127.0.0.1:{0}", port);
        let tcp_listener = TcpListener::bind(endpoint_str.clone())
            .unwrap_or_else(|_| panic!("Failed to bind tcp listener to endpoint:\n {0}", endpoint_str));
        let chat_server = ChatServer {
            tcp_listener,
        };
        println!("Chat server is listening for new connections...");
        chat_server.run();
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
//        let tm_tcp_stream = TmTcpStream { tcp_stream: tcp_stream };
    }
}
