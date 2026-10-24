#![no_std]

pub mod board;
pub mod board_peripherals;
pub mod channel;
pub mod command;
pub use board::*;
pub use board_peripherals::*;
pub use channel::*;
pub use command::*;
// use embedded_hal::delay::DelayNs;

pub mod prelude {
    pub use super::*;
}

pub use ublox::{
    FixedBuffer, Parser, ParserBuilder, ParserError, UbxPacket, UbxPacketMeta,
    mon_ver::MonVer,
    nav_pvt::proto33::NavPvt,
    nav_sat::NavSat,
    proto33::{PacketRef, Proto33},
    rxm_cor::RxmCor,
    sec_sig::{JammingState, SecSig, SpoofingState},
};

// use core::error::Error;
use core::fmt::{self, Display};
use defmt::*;
pub use embassy_embedded_hal::shared_bus::{I2cDeviceError, asynch::i2c::I2cDevice};
pub use embassy_stm32::{
    Peri,
    exti::ExtiInput,
    gpio::{AnyPin, Input, Level, Output, Pin, Pull, Speed},
    i2c::{self, Address, I2c, Master},
    mode::Blocking,
    peripherals::*,
    spi::Spi,
};
pub use embassy_time::{Delay, Duration, Timer, block_for};
pub use embedded_hal_async::i2c::{Error as I2cErrorEmbeddedHal, I2c as I2cAsync};

/////////
pub const MAX_M10S_ADDRESS: u8 = 0x42;
pub const MAX_M10S_NUM_BYTES_HIGH_ADDR: u8 = 0xFD;
pub const MAX_M10S_NUM_BYTES_LOW_ADDR: u8 = 0xFE;
pub const MAX_M10S_DATA_ADDR: u8 = 0xFF;
pub const MAX_M10S_SYNC_BYTE1: u8 = 0xB5;
pub const MAX_M10S_SYNC_BYTE2: u8 = 0x62;

pub type BoardResult<T, BUS> = Result<T, BoardError<BUS>>;

#[derive(Debug)]
pub enum BoardError<BUS> {
    SpiError,
    I2CError(BUS),
    GpsError,
    ChannelError,
    ChannelFrameError,
    Other,
}

// impl<BUS> Error for BoardError<BUS> where BUS: embedded_hal_async::i2c::Error {}

// impl<BUS> Display for BoardError<BUS>
// where
//     BUS: embedded_hal_async::i2c::Error,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             BoardError::SpiError => core::write!(f, "Spi Error Occured"),
//             BoardError::I2CError(_) => core::write!(f, "I2C Error Occured"),
//             _ => core::write!(f, "Some other error occured"),
//         }
//     }
// }

impl<BUS> defmt::Format for BoardError<BUS> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            BoardError::SpiError => defmt::write!(f, "Spi Error Occured"),
            BoardError::I2CError(_) => defmt::write!(f, "I2C Error Occured"),
            _ => defmt::write!(f, "Some other error occured"),
        }
    }
}

// implement from bus error to board error
impl<BUS> From<BUS> for BoardError<BUS>
where
    BUS: embedded_hal_async::i2c::Error,
{
    fn from(value: BUS) -> Self {
        BoardError::I2CError(value)
    }
}

// impl From<I2cError> for BoardError {
//     fn from(value: I2cError) -> Self {
//         match value {
//             _ => Self::I2CError,
//         }
//     }
// }
// pub struct BoardHAL<I2C> {
//     temp_sensor: TempSensor<I2C>,
// }

pub mod helpers {
    use super::*;
    pub async fn i2c_scanner<'a>(mut i2c: I2c<'a, Async, Master>) -> I2c<'a, Async, Master> {
        info!("Scanning I2C bus...");
        let mut found_devices = 0;

        for address in 0x08..=0x77 {
            // We use an empty array [] to perform a zero-length address probe
            match i2c.write(Address::SevenBit(address), &[]).await {
                Ok(_) => {
                    info!("Found device at address: 0x{:02X}", address);
                    found_devices += 1;
                }
                Err(i2c::Error::Nack) => {
                    // No device responded at this address (expected behavior for empty slots)
                }
                Err(e) => {
                    // Handle actual bus errors (e.g., arbitration loss or bus errors)
                    warn!("Error at address 0x{:02X}: {:?}", address, e);
                }
            }
        }

        info!("found devices: {}", found_devices);

        i2c
    }

    pub fn i2c_scanner_blocking<'a>(
        mut i2c: I2c<'a, Blocking, Master>,
    ) -> I2c<'a, Blocking, Master> {
        info!("Scanning I2C bus...");
        let mut found_devices = 0;

        for address in 0x08..=0x77 {
            // We use an empty array [] to perform a zero-length address probe
            match i2c.blocking_write(Address::SevenBit(address), &[]) {
                Ok(_) => {
                    info!("Found device at address: 0x{:02X}", address);
                    found_devices += 1;
                }
                Err(i2c::Error::Nack) => {
                    // No device responded at this address (expected behavior for empty slots)
                }
                Err(e) => {
                    // Handle actual bus errors (e.g., arbitration loss or bus errors)
                    warn!("Error at address 0x{:02X}: {:?}", address, e);
                }
            }
        }

        info!("found devices: {}", found_devices);

        i2c
    }
}
