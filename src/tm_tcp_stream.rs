use std::net::TcpStream;

pub struct TmTcpStream {
    pub tcp_stream: TcpStream,
    pub user_name: String,
}