use log::*;

use crate::{
    atcommands::AtCommand,
    atmodule::ATMoudle,
    subscribe::{NextControlCommand, SubMessage},
};

#[derive(Debug)]
pub enum ResponseType {
    OK,
    URC,
    MESSAGE,
    REPLY,
    MQTTSTAT,
    MQTTPING,
    QMTOPEN,
    QMTCONN,
    QMTSUB,
    ERROR,
    STATUS,
    PUBRESPONSE,
    UNKNOWN,
}

#[derive(Debug)]
pub struct ResponseHandlerResponse {
    pub at_command: AtCommand,
    pub control_command: NextControlCommand,
}

impl ResponseHandlerResponse {
    pub fn new(at_command: AtCommand, control_command: NextControlCommand) -> Self {
        ResponseHandlerResponse {
            at_command,
            control_command,
        }
    }

    pub fn at(at_command: AtCommand) -> Self {
        ResponseHandlerResponse {
            at_command,
            control_command: NextControlCommand::NOOP,
        }
    }

    pub fn control(control_command: NextControlCommand) -> Self {
        ResponseHandlerResponse {
            at_command: AtCommand::NOOP,
            control_command,
        }
    }

    pub fn noop() -> Self {
        ResponseHandlerResponse {
            at_command: AtCommand::NOOP,
            control_command: NextControlCommand::NOOP,
        }
    }
}

#[derive(Debug)]
pub struct ATResponse<'a> {
    pub response_type: ResponseType,
    pub response: &'a str,
    pub response_vec: [&'a str; 2],
}

pub enum ProcessedResponse {
    Passed,
    Failed,
    Noop,
}

impl<'a, 'b, 'c> ATResponse<'a> {
    pub fn new(response_type: ResponseType, response: &'a str) -> ATResponse<'a> {
        ATResponse {
            response_type,
            response,
            response_vec: ["MANA", "MANA"],
        }
    }

    pub fn from_string(response: &'a [u8]) -> ATResponse<'a> {
        if let Ok(response) = std::str::from_utf8(response) {
            let mut responses: Vec<&str> =
                response.split("\r\n").filter(|&x| !x.is_empty()).collect();

            if responses.len() == 0 {
                return ATResponse {
                    response_type: ResponseType::UNKNOWN,
                    response: "MANA",
                    response_vec: ["MANA", "MANA"],
                };
            }

            let response_type = match responses.last().unwrap_or(&"MANA").trim() {
                r if r.starts_with("OK") => ResponseType::OK,
                r if r.starts_with("ERROR") => ResponseType::ERROR,
                r if r.starts_with("+QMTRECV") => ResponseType::MESSAGE,
                r if r.starts_with("+QMTSTAT") => ResponseType::MQTTSTAT,
                r if r.starts_with("+QMTPING") => ResponseType::MQTTPING,
                r if r.starts_with("+QMTOPEN") => ResponseType::QMTOPEN,
                r if r.starts_with("+QMTCONN") => ResponseType::QMTCONN,
                r if r.starts_with("+QMTSUB") => ResponseType::QMTSUB,
                r if r.starts_with("+QMTPUBEX") => ResponseType::PUBRESPONSE,
                r if r.starts_with("+") => ResponseType::URC,
                r if r.starts_with(">") => ResponseType::REPLY,
                _ => ResponseType::UNKNOWN,
            };

            if responses.len() == 1 {
                responses.insert(0, "MANA");
            }

            let responses: [&str; 2] = responses.try_into().unwrap_or(["MANA", "MANA"]);
            let response = responses.first().unwrap_or(&"MANA");

            return ATResponse {
                response_type,
                response,
                response_vec: responses,
            };
        }

        ATResponse {
            response_type: ResponseType::UNKNOWN,
            response: "MANA",
            response_vec: ["MANA", "MANA"],
        }
    }

    pub fn from_bytes(response: &'a [u8]) -> ATResponse<'a> {
        ATResponse::from_string(response)
    }
}

#[derive(Debug)]
pub struct ResponseHandler<'a> {
    pub response: ATResponse<'a>,
}

impl<'a> ResponseHandler<'a> {
    pub fn new(response: ATResponse<'a>) -> Self {
        ResponseHandler { response }
    }

    pub fn handle_sim_stat_response(responses: Vec<&'a str>) -> ProcessedResponse {
        // Processes: +QINISTAT: <state>
        // 0: Initializing
        // 1: SIM Ready
        // 2: SMS Ready
        // 4: Phonebook Ready
        // Output is  sum of the above values
        info!("SIM Status Response: {:?}", responses);

        let simstatus_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if simstatus_split.len() < 2 {
            info!("Invalid SIM Status Response");
            return ProcessedResponse::Failed;
        }

        match simstatus_split[1] {
            "0" => {
                info!("SIM Initializing");
            }
            "1" => {
                info!("SIM Ready");
            }
            "2" => {
                info!("SMS Ready");
            }
            "3" => {
                info!("SIM Ready, SMS Ready");
            }
            "4" => {
                info!("Phonebook Ready");
            }
            "5" => {
                info!("SIM Ready, Phonebook Ready");
            }
            "6" => {
                info!("SMS Ready, Phonebook Ready");
            }
            "7" => {
                info!("SIM Ready, SMS Ready, Phonebook Ready");
            }
            _ => {
                info!("Unknown SIM Status");
                return ProcessedResponse::Failed;
            }
        }

        ProcessedResponse::Passed
    }

    pub fn handle_network_operator_query(responses: Vec<&str>) -> ProcessedResponse {
        // Processes: +COPS: <mode>[,<format>[,<oper>[,<AcT>]]]
        // mode: (integer type) : 0: Automatic, 1: Manual, 2: Deregister from
        // network format: (integer type) : 0: Long alphanumeric, 1: Short
        // alphanumeric, 2: Numeric oper: (string type) : Operator in numeric format
        // AcT: (integer type) : Access technology 0: GSM 7: E-UTRAN (LTE)

        info!("Network Operator Response: {:?}", responses);

        let network_operator_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_operator_split.len() < 2 {
            info!("Invalid Network Operator Response");
            return ProcessedResponse::Failed;
        }

        let network_operator = network_operator_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_operator.len() < 3 {
            info!("Invalid Network Operator Response");
            return ProcessedResponse::Failed;
        }

        let mode = network_operator[0].parse::<u8>().unwrap_or(99);
        let format = network_operator[1].parse::<u8>().unwrap_or(99);
        let operator = network_operator[2];
        let act = network_operator[3].parse::<u8>().unwrap_or(99);

        info!(
            "Mode: {}, Format: {}, Operator: {}, Access Technology: {}",
            mode, format, operator, act
        );

        return ProcessedResponse::Passed;
    }

    pub fn handle_network_strength_query(responses: Vec<&str>) -> ProcessedResponse {
        // Processes: +CSQ: <rssi>,<ber>
        // rssi: 0-31, 99
        // ber: 0-7, 99
        info!("Network Strength Response: {:?}", responses);

        let network_strength_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_strength_split.len() < 2 {
            info!("Invalid Network Strength Response");
            return ProcessedResponse::Failed;
        }

        let network_strength = network_strength_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_strength.len() < 2 {
            info!("Invalid Network Strength Response");
            return ProcessedResponse::Failed;
        }

        let rssi = network_strength[0].parse::<u8>().unwrap_or(99);
        let ber = network_strength[1].parse::<u8>().unwrap_or(99);

        info!("RSSI: {}, BER: {}", rssi, ber);

        ProcessedResponse::Passed
    }

    pub fn handle_network_quality_strength(responses: Vec<&str>) -> ProcessedResponse {
        // Processes: +QNWINFO: <AcT>,<oper>,<band>,<channel>
        // Parameter
        // AcT: (String type): The selected access technology.
        // "No Service"
        // "GSM"
        // "GPRS"
        // "TDD LTE"
        // "FDD LTE"
        // <oper> (String type): The operator in numeric format.
        // <band> (String type): String type. The selected band.
        // "GSM 850"
        // "GSM 900"
        // "GSM 1800"
        // "GSM 1900"
        // "LTE BAND 1"
        // "LTE BAND 2"
        // "LTE BAND 3"
        // "LTE BAND 4"
        // "LTE BAND 5"
        // "LTE BAND 7"
        // "LTE BAND 8"
        // "LTE BAND 20"
        // "LTE BAND 28"
        // "LTE BAND 34"
        // "LTE BAND 38"
        // "LTE BAND 39"
        // "LTE BAND 40"
        // "LTE BAND 41"
        // "LTE BAND 66"
        // <channel> (Integer type): Channel ID.

        info!("Network Quality Response: {:?}", responses);

        let network_quality_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_quality_split.len() < 2 {
            info!("Invalid Network Quality Response");
            return ProcessedResponse::Failed;
        }

        let network_quality = network_quality_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_quality.len() < 4 {
            info!("Invalid Network Quality Response");
            return ProcessedResponse::Failed;
        }

        info!("Network Quality: {:?}", network_quality);

        let act = network_quality[0];
        let oper = network_quality[1];
        let band = network_quality[2];
        let channel = network_quality[3];

        info!(
            "Access Technology: {}, Operator: {}, Band: {}, Channel: {}",
            act, oper, band, channel
        );

        ProcessedResponse::Passed
    }

    pub fn handle_network_registration_query(responses: Vec<&str>) -> ProcessedResponse {
        // Processes: +CREG: <n>,<stat>[,<lac>,<ci>[,<AcT>]]
        // n: (integer type)
        // These just means will/will not receive URCs for network registration
        // 0: Disable network registration unsolicited result code (URC)
        // 1: Enable network registration unsolicited result code (URC)
        // 2: Enable network registration unsolicited result code with location
        // information (URC)

        // stat: (integer type)
        // 0: Not registered, ME is not currently searching a new operator to
        // register to 1: Registered, home network 2: Not registered, but ME is
        // currently searching a new operator to register to 3: Registration denied
        // 4: Unknown 5: Registered, roaming lac: (string type) : 2 bytes location
        // area code in hexadecimal format ci: (string type) :  16-bits GSM  or
        // 20-bits LTE cell ID in hexadecimal format AcT: (integer type) : Access
        // technology 0: GSM 7: E-UTRAN (LTE) <err>: Example:

        info!("Network Registration Response: {:?}", responses);

        let network_registration_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_registration_split.len() < 2 {
            return ProcessedResponse::Failed;
        }

        let network_registration = network_registration_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if network_registration.len() < 2 {
            return ProcessedResponse::Failed;
        }

        let n = network_registration[0].parse::<u8>().unwrap_or(99);
        let stat = network_registration[1].parse::<u8>().unwrap_or(99);

        if network_registration.len() == 2 {
            info!("Network Registration|| N:{:?}, Stat:{:?}", n, stat);

            match n {
                0 => {
                    info!("Disable network registration unsolicited result code (URC)");
                }
                1 => {
                    info!("Enable network registration unsolicited result code (URC)");
                }
                2 => {
                    info!("Enable network registration unsolicited result code with location information (URC)");
                }
                _ => {
                    info!("Unknown Network Registration Status");
                }
            }

            match stat {
                0 => {
                    info!("Not Registered");
                }
                1 => {
                    info!("Registered to Home Network");
                }
                5 => {
                    info!("Registered to Roaming Network");
                }
                _ => {
                    info!("Unknown Network Registration Status");
                }
            }

            return ProcessedResponse::Passed;
        }

        let lac = network_registration[2];
        let ci = network_registration[3];
        let act = network_registration[4].parse::<u8>().unwrap_or(99);

        info!(
            "N: {}, Stat: {}, LAC: {}, CI: {}, ACT: {}",
            n, stat, lac, ci, act
        );

        ProcessedResponse::Passed
    }

    pub fn handle_mqtt_open_command(responses: Vec<&str>) -> ProcessedResponse {
        // Parse response, +QMTOPEN: client_id,result
        // Split on comma, get the second value
        // - `<result>`: Integer type. Result of the Open command execution.
        //     - (-1) Failed to open network
        //     - (0) Network opened successfully
        //     - (1) Wrong parameter
        //     - (2) MQTT identifier is occupied
        //     - (3) Failed to activate PDP
        //     - (4) Failed to parse domain name
        //     - (5) Network connection error

        info!("MQTT Open Response: {:?}", responses);

        let mqtt_open_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if mqtt_open_split.len() < 2 {
            return ProcessedResponse::Failed;
        }

        let mqtt_open = mqtt_open_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        let client_id = mqtt_open[0];
        let result = mqtt_open[1].parse::<i8>().unwrap_or(99);

        info!("Client ID: {}, Result: {}", client_id, result);
        match result {
            -1 => {
                info!("Failed to open network");
            }
            0 => {
                info!("Network opened successfully");
            }
            1 => {
                info!("Wrong parameter");
            }
            2 => {
                info!("MQTT identifier is occupied");
            }
            3 => {
                info!("Failed to activate PDP");
            }
            4 => {
                info!("Failed to parse domain name");
            }
            5 => {
                info!("Network connection error");
            }
            _ => {
                info!("Unknown MQTT Open Result");
            }
        }

        match result {
            0 => ProcessedResponse::Passed,
            _ => ProcessedResponse::Failed,
        }
    }

    pub fn handle_mqtt_conn_command(responses: Vec<&str>) -> ProcessedResponse {
        // - Process: `+QMTCONN:<client_idx>,<result>[,<ret_code>]
        //
        // #### Parameters
        // - `<result>` values:
        //   - 0: Packet sent successfully and ACK received from the server
        //   - 1: Packet retransmission
        //   - 2: Failed to send packet
        // - `<ret_code>`: Integer type, Connection status return code.
        //   - 0: Connection Accepted
        //   - 1: Connection Refused: Unacceptable Protocol Version
        //   - 2: Connection Refused: Identifier Rejected
        //   - 3: Connection Refused: Server Unavailable
        //   - 4: Connection Refused: Bad User Name or Password
        //   - 5: Connection Refused: Not Authorized

        info!("MQTT Connection Response: {:?}", responses);

        let mqtt_conn_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if mqtt_conn_split.len() < 2 {
            return ProcessedResponse::Failed;
        }

        let mqtt_conn = mqtt_conn_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        let client_idx = mqtt_conn[0];
        let result = mqtt_conn[1].parse::<i8>().unwrap_or(99);

        if mqtt_conn.len() == 2 {
            info!("Client ID: {}, Result: {}", client_idx, result);
            match result {
                0 => {
                    info!("Packet sent successfully and ACK received from the server");
                }
                1 => {
                    info!("Packet retransmission");
                }
                2 => {
                    info!("Failed to send packet");
                }
                _ => {
                    info!("Unknown MQTT Connection Result");
                    return ProcessedResponse::Failed;
                }
            }
            return ProcessedResponse::Passed;
        }

        let ret_code = mqtt_conn[2].parse::<i8>().unwrap_or(99);
        info!(
            "Client ID: {}, Result: {}, Ret Code: {}",
            client_idx, result, ret_code
        );

        match ret_code {
            0 => {
                info!("Connection Accepted");
            }
            1 => {
                info!("Connection Refused: Unacceptable Protocol Version");
            }
            2 => {
                info!("Connection Refused: Identifier Rejected");
            }
            3 => {
                info!("Connection Refused: Server Unavailable");
            }
            4 => {
                info!("Connection Refused: Bad User Name or Password");
            }
            5 => {
                info!("Connection Refused: Not Authorized");
            }
            _ => {
                info!("Unknown MQTT Connection Ret Code");
                return ProcessedResponse::Failed;
            }
        }

        ProcessedResponse::Passed
    }

    pub fn handle_subscribe_command<'b>(responses: Vec<&'b str>) -> ProcessedResponse {
        // Process: +QMTSUB: <client_id>,<msg_id>,<result>
        // - `<result>`: Integer type. Result of the Subscribe command execution.
        //   - 0: Sent packet successfully and received ACK from server
        //   - 1: Packet retransmission
        //   - 2: Failed to send packet
        //
        info!("QMTSUB Response: {:?}", responses);

        let responses = responses
            .into_iter()
            .filter(|x| !x.contains("MANA"))
            .collect::<Vec<&str>>();

        if responses.len() == 0 {
            return ProcessedResponse::Failed;
        }

        let response_split = responses
            .first()
            .unwrap_or(&"MANA")
            .split(":")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        if response_split.len() < 2 {
            return ProcessedResponse::Failed;
        }

        let response = response_split[1]
            .split(",")
            .into_iter()
            .map(|x| x.trim())
            .into_iter()
            .collect::<Vec<&str>>();

        let client_id = response[0];
        let msg_id = response[1];
        let result = response[2].parse::<i8>().unwrap_or(99);

        info!(
            "Client ID: {}, Message ID: {}, Result: {}",
            client_id, msg_id, result
        );

        match result {
            0 => {
                info!("Sent packet successfully and received ACK from server");
            }
            1 => {
                info!("Packet retransmission");
            }
            2 => {
                info!("Failed to send packet");
                return ProcessedResponse::Failed;
            }
            _ => {
                info!("Unknown QMTSUB Result");
                return ProcessedResponse::Failed;
            }
        }

        ProcessedResponse::Passed
    }

    pub fn handle_publish_response<'b>(responses: Vec<&'b str>) -> ProcessedResponse {
        info!("PUB Response: {:?}", responses);
        ProcessedResponse::Passed
    }

    pub fn handle_response(&self, prev_command: AtCommand) -> ResponseHandlerResponse {
        info!("Response: {:?} {:?}", self.response.response, prev_command);
        match self.response.response_type {
            ResponseType::OK => {
                info!("OK :{:?}", self.response);
                let passed = match prev_command {
                    AtCommand::SIMInit => ResponseHandler::handle_sim_stat_response(
                        self.response.response_vec.to_vec(),
                    ),

                    AtCommand::NetworkOperatorQuery => {
                        ResponseHandler::handle_network_operator_query(
                            self.response.response_vec.to_vec(),
                        )
                    }

                    AtCommand::NetworkStrengthQuery => {
                        ResponseHandler::handle_network_strength_query(
                            self.response.response_vec.to_vec(),
                        )
                    }
                    AtCommand::NetworkQualityQuery => {
                        ResponseHandler::handle_network_quality_strength(
                            self.response.response_vec.to_vec(),
                        )
                    }
                    AtCommand::NetworkRegistrationQuery => {
                        ResponseHandler::handle_network_registration_query(
                            self.response.response_vec.to_vec(),
                        )
                    }

                    AtCommand::PUBLISH => ProcessedResponse::Passed,

                    AtCommand::QMTCFGVersion
                    | AtCommand::QMTCFGSSLEnable
                    | AtCommand::QMTCFGRecv
                    | AtCommand::QSSLCFGSSLVer
                    | AtCommand::QSSLCFGCipher
                    | AtCommand::QSSLCFGSecLevel
                    | AtCommand::QSSLCFGCACert
                    | AtCommand::QSSLCFGIgnoreInvalid
                    | AtCommand::QSSLCFGSNI => ProcessedResponse::Passed,

                    AtCommand::QMTOPEN | AtCommand::QMTCONN | AtCommand::QMTSUBStart => {
                        ProcessedResponse::Noop
                    }
                    _ => {
                        info!("Unknown Command");
                        ProcessedResponse::Noop
                    }
                };

                match passed {
                    ProcessedResponse::Passed => {
                        info!("Passed");
                        return ResponseHandlerResponse::at(ATMoudle::get_next_command(
                            prev_command,
                        ));
                    }
                    ProcessedResponse::Failed => {
                        info!("Failed");
                        return ResponseHandlerResponse::at(prev_command);
                    }
                    ProcessedResponse::Noop => {
                        info!("Noop");
                        return ResponseHandlerResponse::noop();
                    }
                }
            }

            ResponseType::REPLY => {
                info!("REPLY");
                return ResponseHandlerResponse::at(AtCommand::PUBLISH);
            }

            ResponseType::PUBRESPONSE => {
                info!("Got response for publish message command");
                ResponseHandler::handle_publish_response(self.response.response_vec.to_vec());
                return ResponseHandlerResponse::at(AtCommand::PUBLISHSUCCESS);
            }

            ResponseType::QMTOPEN => {
                ResponseHandler::handle_mqtt_open_command(self.response.response_vec.to_vec());
                return ResponseHandlerResponse::at(ATMoudle::get_next_command(prev_command));
            }

            ResponseType::QMTCONN => {
                ResponseHandler::handle_mqtt_conn_command(self.response.response_vec.to_vec());
                return ResponseHandlerResponse::at(ATMoudle::get_next_command(prev_command));
            }

            ResponseType::QMTSUB => {
                info!("QMTSUB");

                let processed =
                    ResponseHandler::handle_subscribe_command(self.response.response_vec.to_vec());

                match processed {
                    ProcessedResponse::Passed => {
                        info!("Passed");

                        // if prev_command == AtCommand::QMTSUBEnd {
                        //     return ResponseHandlerResponse::control(
                        //         NextControlCommand::STATUSUPDATE,
                        //     );
                        // }

                        return ResponseHandlerResponse::at(ATMoudle::get_next_command(
                            prev_command,
                        ));
                    }
                    ProcessedResponse::Failed => {
                        info!("Failed");
                        return ResponseHandlerResponse::at(prev_command);
                    }
                    ProcessedResponse::Noop => {
                        info!("Noop");
                        return ResponseHandlerResponse::noop();
                    }
                }
            }

            ResponseType::ERROR => {
                info!("ERROR");

                match prev_command {
                    AtCommand::QMTOPEN => {
                        info!("SIM Init Failed");
                        return ResponseHandlerResponse::at(AtCommand::QMTOPEN);
                    }
                    _ => {
                        return ResponseHandlerResponse::noop();
                    }
                }
            }
            ResponseType::STATUS => {
                info!("STATUS");
            }
            ResponseType::URC => {
                info!("URC");
            }
            ResponseType::UNKNOWN => {
                info!("UNKNOWN");
            }
            ResponseType::MESSAGE => {
                let processed = SubMessage::process_received_message(&self.response.response_vec);
                if let Ok(message) = processed {
                    match message.next_control_command {
                        NextControlCommand::STATUSUPDATE => {
                            info!("STATUSUPDATE");
                            return ResponseHandlerResponse::control(
                                NextControlCommand::STATUSUPDATE,
                            );
                        }
                        NextControlCommand::POWEROFF => {
                            info!("POWEROFF");
                            return ResponseHandlerResponse::control(NextControlCommand::POWEROFF);
                        }
                        NextControlCommand::POWERON => {
                            info!("POWERON");
                            return ResponseHandlerResponse::control(NextControlCommand::POWERON);
                        }
                        NextControlCommand::NOOP => {
                            info!("NOOP");
                            return ResponseHandlerResponse::noop();
                        }
                    }
                }

                return ResponseHandlerResponse::noop();
            }
            ResponseType::MQTTSTAT => {
                info!("MQTTSTAT");
            }
            ResponseType::MQTTPING => {
                info!("MQTTPING");
            }
        }

        return ResponseHandlerResponse::noop();
    }
}
