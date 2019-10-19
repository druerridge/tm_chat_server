use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCommand {
    pub name: String,
    pub room: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub command_type: String,
    pub message: String,
}

pub const SEND_MESSAGE: &str = "SendMessage";
pub const GET_USERS: &str = "GetUsers";
pub const SWITCH_ROOM: &str = "SwitchRoom";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub command_type: String,
}