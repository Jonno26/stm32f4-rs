#![no_std]

pub mod board;
pub mod peripherals;
pub use board::*;
use embedded_hal_1::delay::DelayNs;
pub use peripherals::*;

pub mod prelude {
    pub use super::*;
}

pub use ublox::{
    FixedBuffer, Parser, ParserBuilder, ParserError, UbxPacket, UbxPacketMeta,
    mon_ver::MonVer,
    nav_pvt::proto33::NavPvt,
    proto33::{PacketRef, Proto33},
    sec_sig::{JammingState, SecSig, SpoofingState},
};

use core::error::Error;
use core::fmt::{self, Display};
use defmt::*;
use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Pin, Speed},
    i2c::{Error as I2cError, I2c, Master},
    mode::Blocking,
    spi::Spi,
};
use embassy_time::{Delay, Duration, Timer, block_for};

pub type BoardResult<T> = Result<T, BoardError>;

/////////
pub const MAX_M10S_ADDRESS: u8 = 0x42;
pub const MAX_M10S_NUM_BYTES_HIGH_ADDR: u8 = 0xFD;
pub const MAX_M10S_NUM_BYTES_LOW_ADDR: u8 = 0xFE;
pub const MAX_M10S_DATA_ADDR: u8 = 0xFF;
pub const MAX_M10S_SYNC_BYTE1: u8 = 0xB5;
pub const MAX_M10S_SYNC_BYTE2: u8 = 0x62;

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

pub struct MaxM10s<'a> {
    pub i2c: I2c<'a, Blocking, Master>,
    pub parser: Parser<FixedBuffer<2048>>,
}

impl<'a> MaxM10s<'a> {
    pub fn new(i2c: I2c<'a, Blocking, Master>) -> Self {
        let parser: Parser<FixedBuffer<2048>> = ParserBuilder::new()
            .with_protocol::<Proto33>()
            .with_fixed_buffer::<2048>();

        Self { i2c, parser }
    }

    pub fn get_version(&mut self) {
        let mut buf: [u8; 256] = [0; 256];
        self.poll_msg(MonVer);

        loop {
            self.poll_msg(MonVer);

            // let x = self.bytes_available_to_read();
            // bytes_available = u16::from_be_bytes(x);
            // info!("bytes available to read is: {}", bytes_available);
            self.random_address_read(MAX_M10S_DATA_ADDR, &mut buf);

            let mut it = self.parser.consume_ubx(&buf);

            let packet_found = Self::find_packet_in_parser(&mut it);

            if packet_found {
                break;
            }

            block_for(Duration::from_millis(100));
        }
    }

    pub fn get_pvt(&mut self) -> () {
        let mut buf: [u8; 256] = [0; 256];

        // self.poll_msg(NavPvt);

        // let mut bytes_available: u16 = 0;

        // while bytes_available < 1500 {
        //     let x = self.bytes_available_to_read();
        //     bytes_available = u16::from_be_bytes(x);
        //     info!("bytes available to read is: {}", bytes_available);
        //     // block_for(Duration::from_millis(1000));
        // }

        // block_for(Duration::from_millis(1000));

        // register pointer should now be at 0xFF since we already read from 0xFD and 0xFE
        // self.current_address_read(&mut buf);

        // let x = self.bytes_available_to_read();
        // bytes_available = u16::from_be_bytes(x);
        // info!("bytes available to read is: {}", bytes_available);

        loop {
            self.poll_msg(NavPvt);

            self.random_address_read(MAX_M10S_DATA_ADDR, &mut buf);

            let mut it: ublox::UbxParserIter<'_, FixedBuffer<2048>> = self.parser.consume_ubx(&buf);

            let packet_found = Self::find_packet_in_parser(&mut it);

            if packet_found {
                break;
            }

            block_for(Duration::from_millis(100));
        }
    }

    pub fn get_sec_sig_msg(&mut self) {
        let mut buf: [u8; 512] = [0; 512];
        self.poll_msg(SecSig);

        loop {
            self.poll_msg(SecSig);

            // let x = self.bytes_available_to_read();
            // bytes_available = u16::from_be_bytes(x);
            // info!("bytes available to read is: {}", bytes_available);
            self.random_address_read(MAX_M10S_DATA_ADDR, &mut buf);

            let mut it = self.parser.consume_ubx(&buf);

            let packet_found = Self::find_packet_in_parser(&mut it);

            if packet_found {
                break;
            }

            block_for(Duration::from_millis(100));
        }
    }

    pub fn poll_msg<T: UbxPacketMeta>(&mut self, packet: T) {
        let class = <T as UbxPacketMeta>::CLASS;
        let id = <T as UbxPacketMeta>::ID;
        let mut poll_request: [u8; 8] = [
            MAX_M10S_SYNC_BYTE1,
            MAX_M10S_SYNC_BYTE2,
            T::CLASS,
            T::ID,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        let (a, b) = Self::calculate_checksum(&poll_request[2..6]);
        // info!("a is: {:X}, b is: {:X}", a, b);
        poll_request[6] = a;
        poll_request[7] = b;

        // debug!("poll_request is: {:X}", poll_request);

        self.i2c
            .blocking_write(MAX_M10S_ADDRESS, &poll_request)
            .unwrap();
    }

    pub fn calculate_checksum(msg: &[u8]) -> (u8, u8) {
        let mut ck_a: u8 = 0;
        let mut ck_b: u8 = 0;
        for &byte in msg {
            ck_a = ck_a.wrapping_add(byte);
            ck_b = ck_b.wrapping_add(ck_a);
        }
        (ck_a, ck_b)
    }

    /// By default the address pointer is 0xFF (Data stream)
    pub fn current_address_read(&mut self, buf: &mut [u8]) {
        self.i2c.blocking_read(MAX_M10S_ADDRESS, buf).unwrap();
    }

    pub fn random_address_read(&mut self, register: u8, buf: &mut [u8]) {
        self.i2c
            .blocking_write_read(MAX_M10S_ADDRESS, &[register], buf)
            .unwrap();
    }

    /// Master first reads the number of available bytes at the 0xFD and 0xFE before accessing the data at 0xFF
    pub fn bytes_available_to_read(&mut self) -> [u8; 2] {
        let mut r_buf: [u8; 2] = [0; 2];
        let write_buf = [MAX_M10S_NUM_BYTES_HIGH_ADDR];

        // self.i2c
        //     .blocking_write_read(MAX_M10S_ADDRESS, &write_buf, &mut r_buf)
        //     .unwrap();

        self.random_address_read(MAX_M10S_NUM_BYTES_HIGH_ADDR, &mut r_buf);

        r_buf
    }

    fn find_packet_in_parser(it: &mut ublox::UbxParserIter<'_, FixedBuffer<2048>>) -> bool {
        match it.next() {
            Some(Ok(UbxPacket::Proto33(p))) => {
                info!("proto33 packet received");
                Self::handle_packet_proto33(p);
                return true;
            }

            Some(Err(e)) => match e {
                ParserError::InvalidChecksum { expect, got } => {
                    error!(
                        "PARSER ERROR - INVALID CHECKSUM - expected {:02X} but got {:02X}",
                        expect, got
                    );
                    return false;
                }
                ParserError::InvalidField { packet, field } => {
                    error!(
                        "PARSER ERROR - INVALID FIELD - packet: {}, field: {}",
                        packet, field
                    );
                    return false;
                }
                ParserError::InvalidPacketLen {
                    packet,
                    expect,
                    got,
                } => {
                    error!(
                        "PARSER ERROR - INVALID PACKET LENGTH- packet: {}, expected: {}, got: {}",
                        packet, expect, got
                    );
                    return false;
                }
                ParserError::OutOfMemory { required_size } => {
                    error!(
                        "PARSER ERROR - Parser Buffer Too Small - required size: {}",
                        required_size
                    );
                    return false;
                }
            },
            None => {
                // debug!("No more packets to parse - buffer cannot yield another full packet");
                // The internal buffer is now empty
                // break;
                return false;
            }
        }
    }

    fn handle_packet_proto33(p: PacketRef) {
        info!("Received UBX packet");
        match p {
            PacketRef::NavPvt(nav_pvt) => {
                info!("================================");
                info!("Payload length: {} [bytes]", nav_pvt.payload_len());

                match nav_pvt.fix_type() {
                    ublox::GnssFixType::NoFix => info!("No Fix"),
                    ublox::GnssFixType::DeadReckoningOnly => info!("Dead Reckoning only"),
                    ublox::GnssFixType::Fix2D => info!("2D Fix"),
                    ublox::GnssFixType::Fix3D => info!("3D Fix"),
                    ublox::GnssFixType::GPSPlusDeadReckoning => {
                        info!("GNSS + dead reckoning combined")
                    }
                    ublox::GnssFixType::TimeOnlyFix => info!("Time only fix"),
                    _ => info!("Unknown fix type"),
                }

                info!("Ground Speed: {} [m/s]", nav_pvt.ground_speed_2d());
                info!("Latitude: {} [degrees]", nav_pvt.latitude());
                info!("Longitude: {} [degrees]", nav_pvt.longitude());
                info!("Height: {} [m]", nav_pvt.height_msl());
                info!(
                    "# of Satellites used in solution: {}",
                    nav_pvt.num_satellites()
                );
                info!(
                    "UTC Date: {}-{}-{}",
                    nav_pvt.year(),
                    nav_pvt.month(),
                    nav_pvt.day()
                );
                info!(
                    "UTC Time: {}:{}:{}",
                    nav_pvt.hour(),
                    nav_pvt.min(),
                    nav_pvt.sec()
                );
                info!(
                    "Horizontal Accuracy Estimate: {} [mm]",
                    nav_pvt.horizontal_accuracy()
                );
                info!(
                    "Vertical Accuracy Estimate: {} [mm]",
                    nav_pvt.vertical_accuracy()
                );
                info!("Time Accuracy Estimate: {} [ns]", nav_pvt.time_accuracy());
                info!("================================");
            }

            PacketRef::MonVer(mon_ver) => {
                debug!("Hardware Version: {}", mon_ver.hardware_version());
                debug!("Firmware Version: {}", mon_ver.software_version());
            }

            PacketRef::SecSig(sec_sig) => {
                info!(
                    "Jamming detection: {}",
                    sec_sig.sig_sec_flags().jam_det_enabled,
                );
                info!(
                    "Spoofing detection: {}",
                    sec_sig.sig_sec_flags().spf_det_enabled
                );

                match sec_sig.sig_sec_flags().jamming_state {
                    ublox::sec_sig::JammingState::Unknown => info!("Unknown jamming state"),
                    ublox::sec_sig::JammingState::Ok => info!("No jamming indicated"),
                    ublox::sec_sig::JammingState::Warning => {
                        info!("Jamming intereference detected but fix is ok")
                    }
                    ublox::sec_sig::JammingState::Critical => {
                        info!("Jamming intereference detected and no fix")
                    }
                    _ => info!("Warning; Jamming Detected!"),
                }
            }

            _ => (),
        }
    }
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
