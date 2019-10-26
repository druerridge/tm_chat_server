use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::thread;

use crate::connection_listener::ConnectionListener;
use crate::stream_listener::StreamListener;

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
