pub const MAX_PUB_LINE: usize = 30;
pub const AT: &str = "AT";
pub const QMTCFG_VERSION_COMMAND: &str = "AT+QMTCFG=\"version\",0,3";
pub const QMTCFG_SSLENABLE_COMMAND: &str = "AT+QMTCFG=\"SSL\",0,1,0";
pub const QMTCFG_RECV_COMMAND: &str = "AT+QMTCFG=\"recv/mode\",0,0,1";
pub const QSSLCFG_SSLVER_COMMAND: &str = "AT+QSSLCFG=\"sslversion\",0,4";
pub const QSSLCFG_CIPHER_COMMAND: &str = "AT+QSSLCFG=\"ciphersuite\",0,0XFFFF";
pub const QSSLCFG_SECLEVEL_COMMAND: &str = "AT+QSSLCFG=\"seclevel\",0,0";
pub const QSSLCFG_CACERT_COMMAND: &str = "AT+QSSLCFG=\"cacert\",0,\"hive\"";
pub const QSSLCFG_IGNOREINVALID_COMMAND: &str = "AT+QSSLCFG=\"ignoreinvalidcertsign\",0,1";
pub const QSSLCFG_SNI_COMMAND: &str = "AT+QSSLCFG=\"sni\",0,1";
pub const QMTOPEN_COMMAND: &str = "AT+QMTOPEN=0,\"abc.dev.url.com\",8883";
pub const QMTCONN_COMMAND: &str = "AT+QMTCONN=0,\"U2\",\"username\",\"password\"";
pub const QMTSUBSTART_COMMAND: &str = "AT+QMTSUB=0,1,\"SUBONE/start\",0";
pub const QMTSUBEND_COMMAND: &str = "AT+QMTSUB=0,1,\"SUBONE/end\",0";
pub const QMTSUBSTATUS_COMMAND: &str = "AT+QMTSUB=0,1,\"SUBONE/status\",0";

// SIM STATUS
pub const SIM_INIT_COMMAND: &str = "AT+QINISTAT";

// Network Status
pub const NETWORK_REGISTRATION_QUERY: &str = "AT+CREG?";
pub const NETWORK_OPERATOR_QUERY: &str = "AT+COPS?";
pub const NETWORK_STRENGTH_QUERY: &str = "AT+CSQ";
pub const NETWORK_QUALITY_QUERY: &str = "AT+QNWINFO";

// MQTT Connection Status
pub const QMTOPEN_QUERY: &str = "AT+QMTOPEN?";
pub const QMTCONN_QUERY: &str = "AT+QMTCONN?";

pub const QMTDISC_COMMAND: &str = "AT+QMTDISC=0";
pub const QMTCLOSE_COMMAND: &str = "AT+QMTCLOSE=0";

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AtCommand {
    AT,
    QMTCFGVersion,
    QMTCFGSSLEnable,
    QMTCFGRecv,
    QSSLCFGSSLVer,
    QSSLCFGCipher,
    QSSLCFGSecLevel,
    QSSLCFGCACert,
    QSSLCFGIgnoreInvalid,
    QSSLCFGSNI,
    QMTOPEN,
    QMTCONN,
    QMTSUBStart,
    QMTSUBEnd,
    QMTSUBStatus,
    SIMInit,
    NetworkRegistrationQuery,
    NetworkOperatorQuery,
    NetworkStrengthQuery,
    NetworkQualityQuery,
    QMTOPENQuery,
    QMTCONNQuery,
    QMTDISC,
    QMTCLOSE,
    PUBLISH,
    PUBLISHSUCCESS,
    NOOP,
}

pub const STATUS_COMMAND_SEQUENCE: [AtCommand; 6] = [
    AtCommand::AT,
    AtCommand::SIMInit,
    AtCommand::NetworkOperatorQuery,
    AtCommand::NetworkStrengthQuery,
    AtCommand::NetworkQualityQuery,
    AtCommand::NetworkRegistrationQuery,
];

pub const MQTT_CONFIG_COMMAND_SEQUENCE: [AtCommand; 9] = [
    AtCommand::QMTCFGVersion,
    AtCommand::QMTCFGSSLEnable,
    AtCommand::QMTCFGRecv,
    AtCommand::QSSLCFGSSLVer,
    AtCommand::QSSLCFGCipher,
    AtCommand::QSSLCFGSecLevel,
    AtCommand::QSSLCFGCACert,
    AtCommand::QSSLCFGIgnoreInvalid,
    AtCommand::QSSLCFGSNI,
];

pub const MQTT_CONNECTION_COMMAND_SEQUENCE: [AtCommand; 5] = [
    AtCommand::QMTOPEN,
    AtCommand::QMTCONN,
    AtCommand::QMTSUBStart,
    AtCommand::QMTSUBEnd,
    AtCommand::QMTSUBStatus,
];

impl AtCommand {
    pub fn as_str(&self) -> &str {
        match self {
            AtCommand::AT => "AT\r\n",
            AtCommand::QMTCFGVersion => "AT+QMTCFG=\"version\",0,3\r\n",
            AtCommand::QMTCFGSSLEnable => "AT+QMTCFG=\"SSL\",0,1,0\r\n",
            AtCommand::QMTCFGRecv => "AT+QMTCFG=\"recv/mode\",0,0,1\r\n",
            AtCommand::QSSLCFGSSLVer => "AT+QSSLCFG=\"sslversion\",0,4\r\n",
            AtCommand::QSSLCFGCipher => "AT+QSSLCFG=\"ciphersuite\",0,0XFFFF\r\n",
            AtCommand::QSSLCFGSecLevel => "AT+QSSLCFG=\"seclevel\",0,0\r\n",
            AtCommand::QSSLCFGCACert => "AT+QSSLCFG=\"cacert\",0,\"hive\"\r\n",
            AtCommand::QSSLCFGIgnoreInvalid => "AT+QSSLCFG=\"ignoreinvalidcertsign\",0,1\r\n",
            AtCommand::QSSLCFGSNI => "AT+QSSLCFG=\"sni\",0,1\r\n",
            AtCommand::QMTOPEN => "AT+QMTOPEN=0,\"abc.dev.url.com\",8883\r\n",
            AtCommand::QMTCONN => "AT+QMTCONN=0,\"U2\",\"username\",\"password\"\r\n",
            AtCommand::QMTSUBStart => "AT+QMTSUB=0,1,\"SUBONE/start\",0\r\n",
            AtCommand::QMTSUBEnd => "AT+QMTSUB=0,1,\"SUBONE/end\",0\r\n",
            AtCommand::QMTSUBStatus => "AT+QMTSUB=0,1,\"SUBONE/status\",0\r\n",
            AtCommand::SIMInit => "AT+QINISTAT\r\n",
            AtCommand::NetworkRegistrationQuery => "AT+CREG?\r\n",
            AtCommand::NetworkOperatorQuery => "AT+COPS?\r\n",
            AtCommand::NetworkStrengthQuery => "AT+CSQ\r\n",
            AtCommand::NetworkQualityQuery => "AT+QNWINFO\r\n",
            AtCommand::QMTOPENQuery => "AT+QMTOPEN?\r\n",
            AtCommand::QMTCONNQuery => "AT+QMTCONN?\r\n",
            AtCommand::QMTDISC => "AT+QMTDISC=0\r\n",
            AtCommand::QMTCLOSE => "AT+QMTCLOSE=0\r\n",
            AtCommand::PUBLISH => "",
            AtCommand::PUBLISHSUCCESS => "",
            AtCommand::NOOP => "",
        }
    }

    pub fn from_str(command: &str) -> Option<AtCommand> {
        match command {
            r if r.contains("+QMTCFG") => Some(AtCommand::QMTCFGRecv),
            r if r.contains("+QSSLCFG") => Some(AtCommand::QSSLCFGSSLVer),
            r if r.contains("+QMTOPEN") => Some(AtCommand::QMTOPEN),
            r if r.contains("+QMTCONN") => Some(AtCommand::QMTCONN),
            r if r.contains("+QMTSUB") => Some(AtCommand::QMTSUBStart),
            r if r.contains("+QINISTAT") => Some(AtCommand::SIMInit),
            r if r.contains("+CREG") => Some(AtCommand::NetworkRegistrationQuery),
            r if r.contains("+COPS") => Some(AtCommand::NetworkOperatorQuery),
            r if r.contains("+CSQ") => Some(AtCommand::NetworkStrengthQuery),
            r if r.contains("+QNWINFO") => Some(AtCommand::NetworkQualityQuery),
            r if r.contains("+QMTDISC") => Some(AtCommand::QMTDISC),
            r if r.contains("+QMTCLOSE") => Some(AtCommand::QMTCLOSE),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Commander {
    pub command: AtCommand,
}

impl Commander {
    pub fn at() -> Self {
        Commander {
            command: AtCommand::AT,
        }
    }

    // Status (SIM/NETWORK) Query Commands
    pub fn sim_status() -> Self {
        Commander {
            command: AtCommand::SIMInit,
        }
    }

    pub fn query_network_strength() -> Self {
        Commander {
            command: AtCommand::NetworkStrengthQuery,
        }
    }

    pub fn query_network_quality() -> Self {
        Commander {
            command: AtCommand::NetworkQualityQuery,
        }
    }

    pub fn query_network_operator() -> Self {
        Commander {
            command: AtCommand::NetworkOperatorQuery,
        }
    }

    pub fn query_network_registration() -> Self {
        Commander {
            command: AtCommand::NetworkRegistrationQuery,
        }
    }

    // Status (MQTT Connection) Commands
    pub fn query_mqtt_connectoin_open() -> Self {
        Commander {
            command: AtCommand::QMTOPENQuery,
        }
    }

    pub fn query_mqtt_connection_connected() -> Self {
        Commander {
            command: AtCommand::QMTCONNQuery,
        }
    }

    // MQTT + SSL Config
    pub fn config_mqtt_version() -> Self {
        Commander {
            command: AtCommand::QMTCFGVersion,
        }
    }

    pub fn config_mqtt_receive_mode() -> Self {
        Commander {
            command: AtCommand::QMTCFGRecv,
        }
    }

    pub fn config_enable_ssl() -> Self {
        Commander {
            command: AtCommand::QMTCFGSSLEnable,
        }
    }

    pub fn config_ssl_version() -> Self {
        Commander {
            command: AtCommand::QSSLCFGSSLVer,
        }
    }

    pub fn config_ssl_cipher() -> Self {
        Commander {
            command: AtCommand::QSSLCFGCipher,
        }
    }

    pub fn config_ssl_sec_level() -> Self {
        Commander {
            command: AtCommand::QSSLCFGSecLevel,
        }
    }

    pub fn config_ssl_ca_cert() -> Self {
        Commander {
            command: AtCommand::QSSLCFGCACert,
        }
    }

    pub fn config_ssl_ignore_invalid_cert() -> Self {
        Commander {
            command: AtCommand::QSSLCFGIgnoreInvalid,
        }
    }

    pub fn config_ssl_sni() -> Self {
        Commander {
            command: AtCommand::QSSLCFGSNI,
        }
    }

    // MQTT Query command
    pub fn query_mqtt_connection() -> Self {
        Commander {
            command: AtCommand::QMTCONNQuery,
        }
    }

    // MQTT Connection Commands
    pub fn open_mqtt_connection() -> Self {
        Commander {
            command: AtCommand::QMTOPEN,
        }
    }

    pub fn connect_mqtt_client() -> Self {
        Commander {
            command: AtCommand::QMTCONN,
        }
    }

    pub fn subscribe_mqtt_start_topic() -> Self {
        Commander {
            command: AtCommand::QMTSUBStart,
        }
    }

    pub fn subscribe_mqtt_end_topic() -> Self {
        Commander {
            command: AtCommand::QMTSUBEnd,
        }
    }

    pub fn subscribe_mqtt_status_topic() -> Self {
        Commander {
            command: AtCommand::QMTSUBStatus,
        }
    }
}
