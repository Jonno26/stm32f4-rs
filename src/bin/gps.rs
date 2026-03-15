#![no_std]
#![no_main]

pub use stm32f4_rs::prelude::*;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

use ublox::{
    Parser, ParserError, UbxPacket,
    nav_pvt::proto33::NavPvt,
    proto33::{PacketRef, Proto33},
};
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut board = STM32F4::init();

    let m10_reset = board.m10_reset;
    info!("m10 reset pin is: {}", m10_reset.get_output_level());

    let mut i2c = board.i2c1;
    i2c = helpers::i2c_scanner(i2c);

    // UBX NAV-PVT poll request format
    let poll_pvt: [u8; 8] = [
        0xB5, 0x62, // sync chars
        0x01, 0x07, // class = NAV, id = PVT
        0x00, 0x00, // payload length = 0
        0x08, 0x19, // checksum
    ];

    // Initialize the parser
    let mut parser: Parser<ublox::FixedBuffer<256>, Proto33> = Parser::new_fixed();

    let mut r_buf: [u8; 2] = [0; 2];
    let write_buf = [MAX_M10S_NUM_BYTES_HIGH_ADDR];
    let mut buf: [u8; 256] = [0; 256];

    loop {
        i2c.blocking_write(MAX_M10S_ADDRESS, &poll_pvt).unwrap();

        // // Read out how many bytes are available to read
        // i2c.blocking_write_read(MAX_M10S_ADDRESS, &write_buf, &mut r_buf)
        //     .unwrap();

        // let bytes_available = ((r_buf[0] as usize) << 8) | (r_buf[1] as usize);

        // if bytes_available > 1024 {
        // info!("Bytes available to read: {}", bytes_available);

        i2c.blocking_read(MAX_M10S_ADDRESS, &mut buf).unwrap();

        let mut it = parser.consume_ubx(&buf);
        loop {
            match it.next() {
                Some(Ok(UbxPacket::Proto33(p))) => {
                    info!("proto33 packet received");
                    handle_packet_proto33(p);
                }

                Some(Err(e)) => match e {
                    ParserError::InvalidChecksum { expect, got } => {
                        error!(
                            "PARSER ERROR - INVALID CHECKSUM - expected {:02X} but got {:02X}",
                            expect, got
                        );
                    }
                    ParserError::InvalidField { packet, field } => {
                        error!(
                            "PARSER ERROR - INVALID FIELD - packet: {}, field: {}",
                            packet, field
                        );
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
                    }
                    ParserError::OutOfMemory { required_size } => {
                        error!(
                            "PARSER ERROR - Parser Buffer Too Small - required size: {}",
                            required_size
                        );
                    }
                },
                None => {
                    // debug!("No more packets to parse - buffer cannot yield another full packet");
                    // The internal buffer is now empty
                    break;
                }
            }
        }
        // }

        Timer::after_millis(150).await;
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
                ublox::GnssFixType::GPSPlusDeadReckoning => info!("GNSS + dead reckoning combined"),
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
        _ => (),
    }
}
