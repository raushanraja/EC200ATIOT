use esp_idf_svc::hal::uart::{AsyncUartDriver, UartDriver};

use core::fmt::Display;
use core::fmt::Formatter;
const ADDR_DEFAULT: u8 = 0xf8; // Universal address for single-slave environment
const ADDR_MIN: u8 = 0x01;
const ADDR_MAX: u8 = 0xf7;
//
const CMD_READ: u8 = 0x04; // Read the measurement registers
const CMD_RESET: u8 = 0x42; // Reset the energy counter

const CMD_READ_PARAM: u8 = 0x03; // Read the slave parameters
const CMD_WRITE_PARAM: u8 = 0x06; // Write the slave parameters

const PARAM_THRESHOLD: u16 = 0x0001; // Power alarm threshold
const PARAM_ADDR: u16 = 0x0002; // Modbus-RTU address

const REG_COUNT: u16 = 10; // 10 registers in total
                           //
                           //
                           // Define your WriteError and ReadError types
#[derive(Debug, Clone)]
pub struct MyWriteError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct MyReadError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum Error<WriteError, ReadError> {
    CrcMismatch,
    PzemError,
    IllegalAddress,
    WriteError(WriteError),
    ReadError(ReadError),
}
//
impl<WriteError: Display, ReadError: Display> Display for Error<WriteError, ReadError> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Error::CrcMismatch => write!(f, "CRC doesn't match"),
            Error::PzemError => write!(f, "Internal PZEM004T error"),
            Error::IllegalAddress => write!(f, "Illegal address"),
            Error::WriteError(e) => write!(f, "Could not write: {}", e),
            Error::ReadError(e) => write!(f, "Could not read: {}", e),
        }
    }
}
//
// 16-bit cyclic redundancy check (CRC).
fn crc_write(buf: &mut [u8]) {
    let n = buf.len();
    let crc = u16::to_be(crc16::State::<crc16::MODBUS>::calculate(&buf[0..n - 2]));

    buf[n - 2] = (crc >> 8) as u8;
    buf[n - 1] = (crc >> 0) as u8;
}

fn crc_check(buf: &[u8]) -> bool {
    let n = buf.len();
    let crc = u16::from_be(crc16::State::<crc16::MODBUS>::calculate(&buf[0..n - 2]));

    (crc >> 8) as u8 == buf[n - 2] && crc as u8 == buf[n - 1]
}

fn result_convert(buf: &[u8; 25], m: &mut Measurement) {
    m.voltage = (((buf[3] as u16) << 8) | buf[4] as u16) as f32 / 10.0;
    m.current = (((buf[5] as u32) << 8)
        | ((buf[6] as u32) << 0)
        | ((buf[7] as u32) << 24)
        | ((buf[8] as u32) << 16)) as f32
        / 1000.0;
    m.power = (((buf[9] as u32) << 8)
        | ((buf[10] as u32) << 0)
        | ((buf[11] as u32) << 24)
        | ((buf[12] as u32) << 16)) as f32
        / 10.0;
    m.energy = (((buf[13] as u32) << 8)
        | ((buf[14] as u32) << 0)
        | ((buf[15] as u32) << 24)
        | ((buf[16] as u32) << 16)) as f32
        / 1000.0;
    m.frequency = (((buf[17] as u16) << 8) | ((buf[18] as u16) << 0)) as f32 / 10.0;
    m.pf = (((buf[19] as u16) << 8) | ((buf[20] as u16) << 0)) as f32 / 100.0;
    m.alarm = (((buf[21] as u16) << 8) | ((buf[22] as u16) << 0)) != 0;
}

/// Measurement results stored as the 32-bit floating point variables.
#[derive(Debug, Default, Copy, Clone)]
pub struct Measurement {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
    pub energy: f32,
    pub frequency: f32,
    pub pf: f32,
    pub alarm: bool,
}

/// Struct representing a PZEM004T sensor connected to a serial bus.
pub struct Pzem<'a> {
    pub uart: AsyncUartDriver<'a, UartDriver<'a>>,
    pub addr: u8,
}
//
impl<'a> Pzem<'a> {
    /// Creates a new PZEM004T struct, consuming the serial peripheral.
    ///
    /// When omitting the `addr` argument, will use the default general address for a
    /// single-slave environment, namely `0xf8`.
    ///
    /// Can return `Err(Error::IllegalAddress)` if `addr` is not in range of legal addresses `[0x01..0xf8]`.
    pub fn new<'b>(uart: AsyncUartDriver<'a, UartDriver<'a>>) -> Result<Self, bool> {
        let addr = ADDR_DEFAULT;
        if addr != ADDR_DEFAULT && (addr < ADDR_MIN || addr > ADDR_MAX) {
            return Err(false);
        }
        Ok(Self { uart, addr })
    }

    pub async fn communicate(
        &mut self,
        req: &[u8],
        resp: &mut [u8],
    ) -> Result<(), Error<MyWriteError, MyReadError>> {
        let write_result = self.uart.write(&req).await;
        match write_result {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::WriteError(MyWriteError {
                    message: "Error writing to UART".to_string(),
                }))
            }
        }

        // Wait for the response
        let read_result = self.uart.read(resp).await;

        //
        // // First two bytes of the response (slave addr. + function code)
        // // must correspond to the request.
        if resp[0] != req[0] || resp[1] != req[1] {
            return Err(Error::PzemError);
        }
        //
        // Validate CRC
        if !crc_check(&resp) {
            return Err(Error::CrcMismatch);
        }
        match read_result {
            Ok(_) => {}
            Err(_) => {
                return Err(Error::ReadError(MyReadError {
                    message: "Error reading from UART".to_string(),
                }))
            }
        }

        Ok(())
    }
    pub async fn read(
        &mut self,
        m: &mut Measurement,
    ) -> Result<(), Error<MyWriteError, MyReadError>> {
        let mut buf = [
            self.addr,              // Slave address
            CMD_READ,               // Function code: read measurement result
            0,                      // Register address high byte
            0,                      // Register address low byte
            (REG_COUNT >> 8) as u8, // Number of registers to be read.
            (REG_COUNT >> 0) as u8, // Number of registers to be read.
            0,                      // CRC
            0,                      // CRC
        ];

        crc_write(&mut buf);

        // The response: slave address + CMD_RIR + number of bytes + 20 bytes + CRC + CRC
        let mut resp: [u8; 25] = [0; 25];
        self.communicate(&buf, &mut resp).await?;
        result_convert(&resp, m);
        Ok(())
    }
}
