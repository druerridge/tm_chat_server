use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use serde_json::Error;

use crate::commands::{Command, ConnectionCommand, GET_USERS, GetUsersRequest, GetUsersResponse, SEND_MESSAGE, SendMessageRequest, SendMessageResponse, SWITCH_ROOM, SwitchRoomRequest};
use crate::tm_tcp_stream::TmTcpStream;

pub struct StreamListener {
    pub unassigned_streams: Vec<TcpStream>,
    pub unattached_stream_receiver: Receiver<TcpStream>,
    pub tcp_streams_by_room_id: HashMap<RoomId, Vec<TmTcpStream>>,
}

type RoomId = String;

impl StreamListener {
    pub fn run(&mut self) {
        loop {
            self.receive_unattached_streams();

            self.listen_unassigned_streams();

            self.listen_assigned_streams();
        }
    }

    fn parse_send_message(&mut self, user_name: &str, message: &str, room_id: &str) {
        let tm_tcp_streams = self.tcp_streams_by_room_id.get_mut(room_id).expect("Room disappeared"); // TODO: this is handle-able
        let send_message_result: Result<SendMessageRequest, Error> = serde_json::from_str(message);
        if let Ok(send_message_request) = send_message_result {
            let send_message_response: SendMessageResponse = SendMessageResponse {
                command_type: String::from(SEND_MESSAGE),
                message: format!("{0}: {1}", user_name, send_message_request.message),
            };
            let message = serde_json::to_string(&send_message_response).expect("failed to serialize our own struct");
            StreamListener::write_to_room(message.as_str(), tm_tcp_streams);
        } else {
            eprintln!("Error parsing SendMessage command with content: {0}", message);
        }
    }

    fn parse_switch_room(&mut self, user_name: &str, message: &str, room_id: &str) {
        let switch_room_result: Result<SwitchRoomRequest, Error> = serde_json::from_str(message);
        match switch_room_result {
            Ok(switch_room_request) => {
                let mut option_removed_stream = None;
                let tm_tcp_streams = self.tcp_streams_by_room_id.get_mut(room_id).expect("Room disappeared"); // TODO: this is handle-able
                if let Some(index_to_remove) = tm_tcp_streams.iter().position(|e| e.user_name == user_name) {
                    option_removed_stream = Some(tm_tcp_streams.remove(index_to_remove));
                }

                if let Some(removed_stream) = option_removed_stream {
                    if let Some(new_room) = self.tcp_streams_by_room_id.get_mut(switch_room_request.room.as_str()) {
                        new_room.push(removed_stream);
                        println!("Switched rooms");
                    } else {
                        let new_streams = vec![removed_stream];
                        self.tcp_streams_by_room_id.insert(switch_room_request.room, new_streams);
                    }
                }
            }
            Err(e) => eprintln!("Error: {0}\nWhile parsing SwitchRoomRequest from message: {1}", e, message),
        };
    }

    fn parse_get_users(&mut self, user_name: &str, message: &str, users_room_id: &str) {
        let get_users_result: Result<GetUsersRequest, Error> = serde_json::from_str(message);
        if let Ok(get_users_request) = get_users_result {

            let request_room_id = get_users_request.room;
            let requested_room = self.tcp_streams_by_room_id.get(request_room_id.as_str()).expect("Room disappeared"); // TODO: this is handle-able
            let users: Vec<String> = requested_room.iter().map(|tm_tcp_stream| tm_tcp_stream.user_name.clone()).collect();
            let get_users_response: GetUsersResponse = GetUsersResponse{
                command_type: String::from(GET_USERS),
                users,
            };

            let get_users_response_payload = serde_json::to_string(&get_users_response).expect("How do you fuck this up?");
            let users_room = self.tcp_streams_by_room_id.get_mut(users_room_id).expect("Couldn't get the users room"); // This seems unlikely?
            let user_stream = users_room.iter_mut().find(|stream| stream.user_name == user_name).expect("User should still be in his room"); // how would we handle this
            StreamListener::write(&mut user_stream.tcp_stream, get_users_response_payload.as_str());
        }
    }

    fn parse(&mut self, user_name: &str, message: &str, room_id: &str) {
        let command_result: Result<Command, Error> = serde_json::from_str(message);
        match command_result {
            Ok(command) => {
                match command.command_type.as_str() {
                    SEND_MESSAGE => self.parse_send_message(user_name, message, room_id),
                    GET_USERS => self.parse_get_users(user_name, message, room_id),
                    SWITCH_ROOM => self.parse_switch_room(user_name, message, room_id),
                    _ => eprintln!("Received an unknown command type: {0}", command.command_type),
                }
            }
            Err(e) => {
                eprintln!("There was an error: {0}\n While parsing message: {1}", e, message);
            }
        }
    }

    fn listen_assigned_streams(&mut self) {
        let mut to_parse = vec![];
        for entry in &mut self.tcp_streams_by_room_id {
            let room_id = entry.0;
            let tm_tcp_streams: &mut Vec<TmTcpStream> = entry.1.borrow_mut();
            for tm_tcp_stream in &mut tm_tcp_streams.iter_mut() {
                if let Some(inbound_message) = StreamListener::read(&mut tm_tcp_stream.tcp_stream) {
                    to_parse.push((tm_tcp_stream.user_name.clone(), inbound_message.clone(), room_id.clone()));
                }
            }
        }

        for message in to_parse {
            self.parse(message.0.as_str(), message.1.as_str(), message.2.as_str());
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

    fn write(tcp_stream: &mut TcpStream, in_message: &str) {
        let out_message = format!("{}\n", in_message); // maestros-ism here for separating messages in the buffer
        println!("Sending: {}", out_message);
        let _ = tcp_stream.write(out_message.as_bytes()).expect("error writing to tcp stream");
        tcp_stream.flush().expect("Error flushing tcpstream after write");
    }

    fn add_to_room(&mut self, connection_command: ConnectionCommand, tcp_stream: TcpStream) {
        let tm_tcp_stream = TmTcpStream {
            tcp_stream,
            user_name: connection_command.name.clone(),
        };
        match self.tcp_streams_by_room_id.get_mut(connection_command.room.as_str()) {
            Some(tcp_streams) => {
                tcp_streams.push(tm_tcp_stream);
                StreamListener::write_to_room(format!("{0} joined the room", connection_command.name).as_str(), tcp_streams)
            }
            None => {
                let vec = vec![tm_tcp_stream];
                self.tcp_streams_by_room_id.insert(connection_command.room, vec);
            }
        }
    }

    fn write_to_room(message: &str, tm_tcp_streams: &mut Vec<TmTcpStream>) {
        for tm_tcp_stream in tm_tcp_streams {
            StreamListener::write(&mut tm_tcp_stream.tcp_stream, message);
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
