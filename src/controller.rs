use core::fmt;

use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{Gpio3, Gpio8, Gpio9, Output, PinDriver},
    reset::restart,
};
use log::info;

use crate::constants::{AT_RESTART_DELAY, RELAY_CHANGE_DELAY};
use crate::subscribe::NextControlCommand;

pub enum ControllerState {
    ON,
    OFF,
}

#[derive(Debug)]
pub enum RelayControllerError {
    StartError,
    StopError,
}

#[derive(Debug)]
pub enum RelaySuccess {
    StartSuccess,
    StopSuccess,
}

impl fmt::Display for RelayControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelayControllerError::StartError => write!(f, "Failed to start the relay controller"),
            RelayControllerError::StopError => write!(f, "Failed to stop the relay controller"),
        }
    }
}

impl std::error::Error for RelayControllerError {}

pub struct RelayController<'a> {
    pub state: ControllerState,
    pub last_command: Option<NextControlCommand>,
    pub updated_at: Option<u64>,
    pub start: PinDriver<'a, Gpio8, Output>,
    pub stop: PinDriver<'a, Gpio9, Output>,
    pub at_restart: PinDriver<'a, Gpio3, Output>,
}

impl<'a, 'b> RelayController<'a> {
    pub fn new(
        state: ControllerState,
        last_command: Option<NextControlCommand>,
        updated_at: Option<u64>,
        start: PinDriver<'a, Gpio8, Output>,
        stop: PinDriver<'a, Gpio9, Output>,
        at_restart: PinDriver<'a, Gpio3, Output>,
    ) -> Self {
        RelayController {
            state,
            last_command,
            updated_at,
            start,
            stop,
            at_restart,
        }
    }

    pub fn start(&mut self) -> Result<RelaySuccess, RelayControllerError> {
        match self.state {
            ControllerState::ON => {
                info!("Device is already ON");
                return Ok(RelaySuccess::StartSuccess);
            }
            ControllerState::OFF => {
                info!("Starting the Device");
            }
        }

        if self.start.set_high().is_ok() {
            Ets::delay_ms(RELAY_CHANGE_DELAY);
            if self.start.set_low().is_ok() {
                self.state = ControllerState::ON;
                return Ok(RelaySuccess::StartSuccess);
            }
        }

        Err(RelayControllerError::StartError)
    }

    pub fn stop(&mut self) -> Result<RelaySuccess, RelayControllerError> {
        match self.state {
            ControllerState::OFF => {
                info!("Device is already OFF");
                return Ok(RelaySuccess::StopSuccess);
            }
            ControllerState::ON => {
                info!("Stopping the Device");
            }
        }

        if self.stop.set_low().is_ok() {
            Ets::delay_ms(RELAY_CHANGE_DELAY);
            if self.stop.set_high().is_ok() {
                self.state = ControllerState::OFF;
                return Ok(RelaySuccess::StopSuccess);
            }
        }

        Err(RelayControllerError::StopError)
    }

    pub fn at_module_restart(&mut self) -> Result<RelaySuccess, RelayControllerError> {
        info!("Restarting the AT Module");
        if self.at_restart.set_high().is_ok() {
            Ets::delay_ms(AT_RESTART_DELAY);
            if self.at_restart.set_low().is_ok() {
                info!("AT Module Restarted");
                return Ok(RelaySuccess::StartSuccess);
            }
        }
        Err(RelayControllerError::StartError)
    }

    pub fn set_state(
        &mut self,
        state: ControllerState,
    ) -> Result<RelaySuccess, RelayControllerError> {
        match state {
            ControllerState::ON => self.start(),
            ControllerState::OFF => self.stop(),
        }
    }

    pub fn status(&self) -> String {
        match self.state {
            ControllerState::ON => "STR".to_string(),
            ControllerState::OFF => "STP".to_string(),
        }
    }
}
