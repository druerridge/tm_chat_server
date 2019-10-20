use std::hash::{Hash, Hasher};
use std::net::TcpStream;

pub struct TmTcpStream {
    pub tcp_stream: TcpStream,
    pub user_name: String,
}

impl PartialEq for TmTcpStream {
    fn eq(&self, other: &TmTcpStream) -> bool {
        if let Ok(my_peer_addr) = self.tcp_stream.peer_addr() {
            if let Ok(other_peer_addr) = other.tcp_stream.peer_addr() {
                return my_peer_addr == other_peer_addr;
            }
        }

        false
    }
}

impl Eq for TmTcpStream {}

impl Hash for TmTcpStream {
    fn hash<H:Hasher>(&self, state: &mut H) {
        if let Ok(my_peer_addr) = self.tcp_stream.peer_addr() {
            my_peer_addr.hash(state);
        }
    }
}