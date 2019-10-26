use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;

pub struct ConnectionListener {
    pub tcp_listener: TcpListener,
    pub unattached_stream_sender: Sender<TcpStream>,
}

impl ConnectionListener {
    pub fn run(self) {
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