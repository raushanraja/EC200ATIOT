use std::i32;

use log::info;

#[derive(Debug, Clone, Copy)]
pub enum NextControlCommand {
    STATUSUPDATE,
    POWEROFF,
    POWERON,
    NOOP,
}

pub struct SubMessage<'a> {
    pub client_id: i32,
    pub msg_id: i32,
    pub topic: &'a str,
    pub payload_len: Option<i32>,
    pub payload: Option<&'a str>,
    pub next_control_command: NextControlCommand,
}

impl<'a> SubMessage<'a> {
    pub fn new(
        client_id: i32,
        msg_id: i32,
        topic: &'a str,
        payload_len: Option<i32>,
        payload: Option<&'a str>,
        control_command: NextControlCommand,
    ) -> Self {
        SubMessage {
            client_id,
            msg_id,
            topic,
            payload_len,
            payload,
            next_control_command: control_command,
        }
    }

    pub fn process_received_message(message: &[&'a str]) -> Result<Self, bool> {
        let updated_message = message
            .iter()
            .filter(|x| !x.contains("MANA"))
            .collect::<Vec<&&str>>();

        info!("Processing Received Message: {:?}", message);
        let recv_split = match updated_message.first() {
            Some(first_message) => first_message.split(':').collect::<Vec<&str>>(),
            None => {
                return Err(false);
            }
        };

        if recv_split.len() == 0 {
            return Err(false);
        }

        let message_parts = recv_split[1]
            .split(",")
            .map(|x| x.trim())
            .collect::<Vec<&str>>();

        if message_parts.len() < 3 {
            info!("Invalid Message");
            return Err(false);
        }

        let client_id: i32 = match message_parts[0].parse::<i32>() {
            Ok(id) => id,
            Err(_) => return Err(false),
        };

        let msg_id: i32 = match message_parts[1].parse::<i32>() {
            Ok(id) => id,
            Err(_) => return Err(false),
        };

        let topic = message_parts[2];

        let next_command = match topic.replace("\"", "").as_str() {
            "SUBONE/start" => NextControlCommand::POWERON,
            "SUBONE/end" => NextControlCommand::POWEROFF,
            "SUBONE/status" => NextControlCommand::STATUSUPDATE,
            _ => NextControlCommand::NOOP,
        };

        let mut message = SubMessage::new(client_id, msg_id, topic, None, None, next_command);

        if message_parts.len() < 3 {
            info!("Invalid Message");
            return Err(false);
        }

        if message_parts.len() == 3 {
            info!(
                "Client ID: {}, Message ID: {}, Topic: {}",
                client_id, msg_id, topic
            );
            return Err(false);
        }

        let payload_lenght = message_parts[3];
        let payload = message_parts[4];

        info!(
            "Client ID: {}, Message ID: {}, Topic: {}, Payload Length: {}, Payload: {}, next_control_command: {:?}",
            client_id, msg_id, topic, payload_lenght, payload, next_command
        );

        message.payload_len = Some(payload_lenght.parse().unwrap());
        message.payload = Some(payload);

        return Ok(message);
    }
}
