use log::info;

use crate::at::Messages;
use crate::atcommands::{
    AtCommand, Commander, MQTT_CONFIG_COMMAND_SEQUENCE, MQTT_CONNECTION_COMMAND_SEQUENCE,
    STATUS_COMMAND_SEQUENCE,
};

#[derive(Debug, Clone)]
pub enum MouduleState {
    INIT,
    STARTED,
    CONNECTING,
    CONNECTED,
    DISCONNECTED,
}

#[derive(Debug, Clone)]
pub enum MoudleEvent {
    INIT,
    STATUS,
    CONFIG,
    CONNECT,
    PUBLISH,
}

#[derive(Debug, Clone)]
pub enum PublishState {
    INIT,
    PUBLISHING,
    PUBLISHED,
    ERROR,
}

#[derive(Debug, Clone)]
pub struct ATMoudle {
    pub state: MouduleState,
    pub event: MoudleEvent,
    pub publish_state: PublishState,
    pub publish_message: Option<String>,
    pub messages: Vec<Messages>,
    pub command: Commander,
}

impl ATMoudle {
    pub fn new() -> Self {
        ATMoudle {
            state: MouduleState::INIT,
            event: MoudleEvent::INIT,
            publish_state: PublishState::INIT,
            publish_message: None,
            messages: Vec::new(),
            command: Commander::at(),
        }
    }

    pub fn new_with_state(state: MouduleState) -> Self {
        ATMoudle {
            state,
            event: MoudleEvent::INIT,
            publish_state: PublishState::INIT,
            publish_message: None,
            messages: Vec::new(),
            command: Commander::at(),
        }
    }

    pub fn set_state(&mut self, state: MouduleState) {
        self.state = state;
    }

    pub fn set_event(&mut self) {
        let event = ATMoudle::get_event_type(self.command.command.clone());
        self.event = event;
    }

    pub fn set_publish_state(&mut self, publish_state: PublishState, message: Option<String>) {
        info!("Setting publish state");
        self.publish_state = publish_state;

        if message.is_some() {
            info!("Setting publish message, deleting old message");
            self.publish_message = message.clone();
            let removed = self.messages.pop();
            let copied = removed.clone().unwrap();
            if removed.is_some_and(|m| m.message == message.unwrap()) {
                info!("Message removed successfully from the list {:?}", copied);
                self.messages.push(copied);
            }
        }
    }

    pub fn set_publish_message(&mut self, message: Option<String>) {
        info!("Setting publish message");
        self.publish_message = message.clone();
        self.messages.push(Messages {
            message: message.clone().unwrap(),
            length: message.unwrap().len(),
        });
    }

    pub fn get_event_type(command: AtCommand) -> MoudleEvent {
        match command {
            AtCommand::AT
            | AtCommand::SIMInit
            | AtCommand::NetworkStrengthQuery
            | AtCommand::NetworkQualityQuery
            | AtCommand::NetworkOperatorQuery
            | AtCommand::NetworkRegistrationQuery => MoudleEvent::STATUS,

            AtCommand::QMTCFGVersion
            | AtCommand::QMTCFGSSLEnable
            | AtCommand::QMTCFGRecv
            | AtCommand::QSSLCFGSSLVer
            | AtCommand::QSSLCFGCipher
            | AtCommand::QSSLCFGSecLevel
            | AtCommand::QSSLCFGCACert
            | AtCommand::QSSLCFGIgnoreInvalid
            | AtCommand::QSSLCFGSNI => MoudleEvent::CONFIG,

            AtCommand::QMTOPEN
            | AtCommand::QMTCONN
            | AtCommand::QMTSUBStart
            | AtCommand::QMTSUBEnd
            | AtCommand::QMTSUBStatus => MoudleEvent::CONNECT,

            _ => MoudleEvent::PUBLISH,
        }
    }

    pub fn get_next_command(command: AtCommand) -> AtCommand {
        let event = ATMoudle::get_event_type(command.clone());
        match event {
            MoudleEvent::STATUS => {
                let index = STATUS_COMMAND_SEQUENCE
                    .iter()
                    .position(|r| -> bool { *r == command });

                match index {
                    Some(i) => {
                        if i < STATUS_COMMAND_SEQUENCE.len() - 1 {
                            return STATUS_COMMAND_SEQUENCE[i + 1];
                        }
                        return MQTT_CONFIG_COMMAND_SEQUENCE[0];
                    }
                    None => return AtCommand::NOOP,
                }
            }
            MoudleEvent::CONFIG => {
                let index = MQTT_CONFIG_COMMAND_SEQUENCE
                    .iter()
                    .position(|r| -> bool { *r == command });

                match index {
                    Some(i) => {
                        if i < MQTT_CONFIG_COMMAND_SEQUENCE.len() - 1 {
                            return MQTT_CONFIG_COMMAND_SEQUENCE[i + 1];
                        }
                        return MQTT_CONNECTION_COMMAND_SEQUENCE[0];
                    }
                    None => return AtCommand::NOOP,
                }
            }
            MoudleEvent::CONNECT => {
                let index = MQTT_CONNECTION_COMMAND_SEQUENCE
                    .iter()
                    .position(|r| -> bool { *r == command });

                match index {
                    Some(i) => {
                        if i < MQTT_CONNECTION_COMMAND_SEQUENCE.len() - 1 {
                            return MQTT_CONNECTION_COMMAND_SEQUENCE[i + 1];
                        }
                        return AtCommand::NOOP;
                    }
                    None => return AtCommand::NOOP,
                }
            }

            MoudleEvent::PUBLISH => return AtCommand::PUBLISHSUCCESS,
            _ => AtCommand::NOOP,
        }
    }
}
