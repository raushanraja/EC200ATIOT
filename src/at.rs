use std::sync::{Arc, Mutex};

use esp_idf_svc::hal::gpio::{
    Gpio10, Gpio11, Gpio3, Gpio5, Gpio6, Gpio8, Gpio9, Output, PinDriver,
};
use esp_idf_svc::hal::reset::restart;
use esp_idf_svc::hal::uart::{AsyncUartDriver, UartDriver};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::{hal::peripherals::Peripherals, hal::uart, hal::uart::config};
use log::*;

use crate::atcommands::AtCommand;
use crate::atcommands::Commander;
use crate::atmodule::{ATMoudle, MouduleState, PublishState};
use crate::atres::{ATResponse, ResponseHandler};
use crate::controller::{ControllerState, RelayController};
use crate::emon;
use crate::subscribe::NextControlCommand;

const BAUDRATE: u32 = 115200;
const PZEMBAUDRATE: u32 = 9600;

pub enum AtReplyTopic {
    START,
    STOP,
    STATUS,
    POWER,
}

impl AtReplyTopic {
    pub fn topic(&self) -> &'static str {
        match self {
            AtReplyTopic::START => "RTONE/start",
            AtReplyTopic::STOP => "RTONE/stop",
            AtReplyTopic::STATUS => "RTONE/status",
            AtReplyTopic::POWER => "RTONE/Power",
        }
    }
}

fn init_uart<'a>() -> Result<
    (
        AsyncUartDriver<'a, uart::UartDriver<'a>>,
        PinDriver<'a, Gpio8, Output>,
        PinDriver<'a, Gpio9, Output>,
        AsyncUartDriver<'a, uart::UartDriver<'a>>,
        PinDriver<'a, Gpio3, Output>,
    ),
    EspError,
> {
    let peripherals = Peripherals::take()?;
    let tx = peripherals.pins.gpio5;
    let rx = peripherals.pins.gpio6;
    let at_restart = peripherals.pins.gpio3;
    let start = PinDriver::output(peripherals.pins.gpio8).expect("Failed to initialize start pin");
    let mut stop =
        PinDriver::output(peripherals.pins.gpio9).expect("Failed to initialize stop pin");
    let at_restart = PinDriver::output(at_restart).expect("Failed to initialize reset pin");

    match stop.set_high() {
        Ok(_) => {
            info!("Stop pin set high Initial");
        }
        Err(e) => {
            info!("Error setting stop pin high {:?}", e);
        }
    }

    let pzemrx = peripherals.pins.gpio10;
    let pzemtx = peripherals.pins.gpio11;

    info!("Creating UART ");

    let config = config::Config::new().baudrate(Hertz(BAUDRATE));
    let async_uart_driver = AsyncUartDriver::new(
        peripherals.uart1,
        tx,
        rx,
        Option::<Gpio5>::None,
        Option::<Gpio6>::None,
        &config,
    );

    let config = config::Config::new().baudrate(Hertz(PZEMBAUDRATE));
    let serial = AsyncUartDriver::new(
        peripherals.uart2,
        pzemtx,
        pzemrx,
        Option::<Gpio10>::None,
        Option::<Gpio11>::None,
        &config,
    );

    Ok((async_uart_driver?, start, stop, serial?, at_restart))
}

pub struct AT<'a> {
    uart: AsyncUartDriver<'a, UartDriver<'a>>,
    module: Arc<Mutex<ATMoudle>>,
    relaycontroller: Arc<Mutex<RelayController<'a>>>,
    pzem: Arc<Mutex<emon::Pzem<'a>>>,
}

#[derive(Debug, Clone)]
pub struct Messages {
    pub message: String,
    pub length: usize,
}

impl AT<'_> {
    pub fn new() -> Self {
        let (uart, start, stop, serial, at_restart) = init_uart().unwrap();
        let module = Arc::new(Mutex::new(ATMoudle::new()));
        let pzem = emon::Pzem::new(serial).unwrap();
        let relaycontroller = Arc::new(Mutex::new(RelayController::new(
            ControllerState::OFF,
            None,
            None,
            start,
            stop,
            at_restart,
        )));
        let pzem = Arc::new(Mutex::new(pzem));
        AT {
            uart,
            module,
            relaycontroller,
            pzem,
        }
    }

    pub async fn publish<'a>(&self, message: &str, topic: AtReplyTopic) {
        let command = format!(
            "AT+QMTPUBEX=0,1,0,0,\"{}\",{}\r\n",
            topic.topic(),
            message.len()
        );

        info!("Command: {:?}", command);

        if let Ok(mut module) = self.module.try_lock() {
            match module.publish_state {
                PublishState::INIT | PublishState::PUBLISHED => {
                    module.set_publish_message(Some(message.to_string()));
                    module.set_publish_state(PublishState::PUBLISHING, None);
                    if self.uart.write(command.as_bytes()).await.is_err() {
                        if self.uart.wait_tx_done().await.is_ok() {
                            info!("Message sent successfully");
                        }
                    }
                    return;
                }
                PublishState::PUBLISHING => {
                    info!("Message already publishing");
                    return;
                }
                _ => {
                    info!(
                        "Message not published current state: {:?}",
                        module.publish_state
                    );
                    return;
                }
            }
        } else {
            info!("Module is locked, skipping publish");
        }
    }

    pub async fn send_serial_message(&self, message: String) {
        if let Ok(mut module) = self.module.try_lock() {
            module.command = Commander {
                command: AtCommand::PUBLISH,
            };
            let _ = self.uart.write(message.as_bytes()).await;
        } else {
            info!("Module is locked, skipping send_serial");
        }
    }

    pub async fn send_serial<'a>(&self, command: Commander) {
        if let Ok(mut module) = self.module.try_lock() {
            module.command = command;
            let _ = self.uart.write(&module.command.command.as_bytes()).await;
        } else {
            info!("Module is locked, skipping send_serial");
        }
    }

    pub async fn read_serail<'a>(&self) {
        loop {
            let mut buffer = [0u8; 256];
            if let Ok(len) = self.uart.read(&mut buffer).await {
                if len == 0 {
                    continue;
                }
                info!(
                    "Bytes {:?}",
                    std::str::from_utf8(&buffer[..len]).unwrap_or("Error Reading bytes")
                );
                let response = ATResponse::from_bytes(&buffer[..len]);
                let command = self.module.lock().unwrap().command.command.clone();
                let next_atcommands = ResponseHandler::new(response).handle_response(command);
                info!("Next Command {:?}", next_atcommands);

                match next_atcommands.control_command {
                    NextControlCommand::POWERON => {
                        let mut relaycontroller = self.relaycontroller.lock().unwrap();
                        let _ = relaycontroller.start();
                        let status = relaycontroller.status();
                        self.sendstatus(Some(String::from(status)), AtReplyTopic::START)
                            .await;
                    }
                    NextControlCommand::POWEROFF => {
                        let mut relaycontroller = self.relaycontroller.lock().unwrap();
                        let _ = relaycontroller.stop();
                        let status = relaycontroller.status();
                        self.sendstatus(Some(String::from(status)), AtReplyTopic::STOP)
                            .await;
                    }
                    NextControlCommand::STATUSUPDATE => {
                        let relaycontroller = self.relaycontroller.lock().unwrap();
                        let status = relaycontroller.status();
                        self.sendstatus(Some(status), AtReplyTopic::STATUS).await;
                    }
                    _ => {}
                }

                match next_atcommands.at_command {
                    AtCommand::NOOP => {
                        info!("NOOP");
                    }
                    AtCommand::PUBLISH => {
                        info!("Publishing message");
                        let message = self.module.lock().unwrap().publish_message.clone();
                        self.send_serial_message(message.unwrap()).await;
                    }
                    AtCommand::PUBLISHSUCCESS => {
                        info!("Publishing message success");
                        self.module
                            .lock()
                            .unwrap()
                            .set_publish_state(PublishState::PUBLISHED, None);
                    }
                    _ => {
                        if next_atcommands.at_command == AtCommand::QMTSUBEnd {
                            self.module
                                .lock()
                                .unwrap()
                                .set_state(MouduleState::CONNECTED);
                        }

                        self.send_serial(Commander {
                            command: next_atcommands.at_command,
                        })
                        .await;
                    }
                }
            }
        }
    }

    pub async fn sendstatus<'a>(&self, message: Option<String>, topic: AtReplyTopic) {
        info!("Sending status message {:?}", message);
        let mut buffer = emon::Measurement::default();
        let current_publish_state = self.module.try_lock();

        if current_publish_state.is_err() {
            info!("Module is locked");
            return;
        }
        let current_publish_state = current_publish_state.unwrap().publish_state.clone();
        log::info!("Current publish state: {:?}", current_publish_state);

        match current_publish_state {
            PublishState::PUBLISHING => {
                info!("Message already publishing");
                return;
            }

            _ => {}
        }

        match message {
            Some(message) => {
                self.publish(&message, topic).await;
                return;
            }
            None => match self.pzem.lock().unwrap().read(&mut buffer).await {
                Ok(_) => {
                    let message = format!(
                        "{},{},{},{},{},{}",
                        buffer.voltage,
                        buffer.current,
                        buffer.power,
                        buffer.energy,
                        buffer.frequency,
                        buffer.pf
                    );
                    self.publish(&message, AtReplyTopic::POWER).await;
                }
                Err(e) => {
                    info!("Error reading from PZEM {:?}", e);
                }
            },
        }
    }

    pub async fn check_at<'a>(&self) -> Result<bool, EspError> {
        let mut started = false;
        while !started {
            info!("Checking AT");
            self.uart.write(AtCommand::AT.as_bytes()).await.unwrap();
            self.uart.wait_tx_done().await.unwrap();
            let mut buffer = [0u8; 256];
            let len = self.uart.read(&mut buffer).await.unwrap();
            if len == 6 && &buffer[..len] == b"\r\nOK\r\n" {
                started = true;
            }
            let mut module = self.module.lock().unwrap();
            module.set_state(MouduleState::STARTED);
        }
        Ok(started)
    }

    pub fn restart<'a>(&self) {
        info!("Restarting AT Module");
        if let Ok(mut controller) = self.relaycontroller.try_lock() {
            let _ = controller.at_module_restart();
            restart();
        }
    }

    pub async fn init<'a>(&self) {
        let command = Commander::sim_status();
        let mut module = self.module.lock().unwrap();
        module.command = command.clone();
        module.set_event();
        self.uart.write(command.command.as_bytes()).await.unwrap();
        self.uart.wait_tx_done().await.unwrap();
    }
}
