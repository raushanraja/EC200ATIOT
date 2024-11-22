//! MQTT asynchronous client example which subscribes to an internet MQTT server and then sends
//! and receives events in its own topic.

pub mod at;
pub mod atcommands;
pub mod atmodule;
pub mod atres;
pub mod constants;
pub mod controller;
pub mod emon;
pub mod subscribe;

use core::pin::pin;
use core::time::Duration;

use at::AT;
use constants::{ATREAD, ATRESTART, ATSTATUS};
use embassy_futures::select::{select, select3, Either3};

use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::sys::{esp_log_level_set, EspError};
use esp_idf_svc::timer::{EspAsyncTimer, EspTimerService};

use log::*;

async fn run(
    timer_one: &mut EspAsyncTimer,
    timer_two: &mut EspAsyncTimer,
    restart_timer: &mut EspAsyncTimer,
) -> Result<(), EspError> {
    info!("About to start the MQTT client");
    let at = AT::new();
    let _ = at.check_at().await;
    at.init().await;

    let delay: Delay = Default::default();

    info!("AT Moudle UART Connection established");

    let res = select3(
        pin!(async {
            loop {
                let _ = timer_one.after(Duration::from_millis(ATSTATUS)).await;
                at.sendstatus(None, at::AtReplyTopic::STATUS).await;
            }
        }),
        pin!(async {
            loop {
                info!("Waiting for  Reading message");
                let _ = timer_two.after(Duration::from_millis(ATREAD)).await;
                at.read_serail().await;
            }
        }),
        pin!(async {
            loop {
                let _ = restart_timer.after(Duration::from_millis(ATRESTART)).await;
                at.restart();
            }
        }),
    )
    .await;

    match res {
        Either3::First(_) => Ok(()),
        Either3::Second(_) => Ok(()),
        Either3::Third(_) => Ok(()),
    }
}

fn main() {
    unsafe {
        use std::ffi::CString;
        let tag = CString::new("*").unwrap();
        if !constants::DEUBGLOGS {
            esp_log_level_set(tag.as_ptr(), 0);
        }
    }

    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Starting ATController");
    let timer_service = EspTimerService::new().unwrap();

    let delay: Delay = Default::default();
    delay.delay_ms(10000);

    let mut timer_one = timer_service.timer_async().unwrap();
    let mut timer_two = timer_service.timer_async().unwrap();
    let mut restart_timer = timer_service.timer_async().unwrap();

    block_on(async {
        loop {
            let _ = run(&mut timer_one, &mut timer_two, &mut restart_timer).await;
        }
    })
}
