//Files for all the command in the project

use serde::Serialize;

//Command to get the user' input from the chat
#[derive(Debug, Serialize)]
pub enum Sender {
    User,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub message: String,
    pub sender: Sender,
}

#[tauri::command]
pub fn get_message(message: &str) -> Message {
    let message = Message {
        message: message.to_string(),
        sender: Sender::User,
    };
    println!("Received message from frontend: {:#?}", message);
    return message;
}
