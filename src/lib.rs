#![no_std]

pub mod board;
pub mod peripherals;
pub use board::*;
pub use peripherals::*;

pub mod prelude {
    pub use super::*;
}

use core::error::Error;
use core::fmt::{self, Display};
use defmt::*;
use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    i2c::{Error as I2cError, I2c, Master},
    mode::Blocking,
    spi::Spi,
};

pub type BoardResult<T> = Result<T, BoardError>;

/////////
pub const MAX_M10S_ADDRESS: u8 = 0x42;

#[derive(Debug)]
pub enum BoardError {
    SpiError,
    I2CError,
    Other,
}

impl Error for BoardError {}

impl Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardError::SpiError => core::write!(f, "Spi Error Occured"),
            BoardError::I2CError => core::write!(f, "I2C Error Occured"),
            _ => core::write!(f, "Some other error occured"),
        }
    }
}

impl defmt::Format for BoardError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            BoardError::SpiError => defmt::write!(f, "Spi Error Occured"),
            BoardError::I2CError => defmt::write!(f, "I2C Error Occured"),
            _ => defmt::write!(f, "Some other error occured"),
        }
    }
}

impl From<I2cError> for BoardError {
    fn from(value: I2cError) -> Self {
        match value {
            _ => Self::I2CError,
        }
    }
}
pub struct BoardHAL<'a> {
    temp_sensor: TempSensor<'a>,
}

pub mod helpers {
    use super::*;
    pub fn i2c_scanner<'a>(mut i2c: I2c<'a, Blocking, Master>) -> I2c<'a, Blocking, Master> {
        info!("Scanning I2C bus...");
        let mut found_devices = 0;

        for addr in 0..=127 {
            match i2c.blocking_write(addr, &[]) {
                Ok(_) => {
                    info!("Found device at address: 0x{:02x}", addr);
                    found_devices += 1;
                }
                Err(e) => {
                    // error!("error was: {}", e);
                    // No acknowledgment received, so no device is present at this address.
                    // Depending on the HAL, some specific errors might need handling,
                    // but a generic error usually means NACK
                }
            }
        }
        info!("I2C scan complete. {} devices found.", found_devices);

        i2c
    }
}
