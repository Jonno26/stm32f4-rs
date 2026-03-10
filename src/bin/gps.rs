#![no_std]
#![no_main]

pub use stm32f4_rs::prelude::*;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

use ublox::{
    Parser, UbxPacket,
    proto31::{PacketRef, Proto31},
};
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut board = STM32F4::init();

    let m10_reset = board.m10_reset;
    info!("m10 reset pin is: {}", m10_reset.get_output_level());

    let mut i2c = board.i2c1;
    i2c = helpers::i2c_scanner(i2c);

    // UBX NAV-PVT poll request
    let poll_pvt: [u8; 8] = [
        0xB5, 0x62, // sync chars
        0x01, 0x07, // class = NAV, id = PVT
        0x00, 0x00, // payload length = 0
        0x08, 0x19, // checksum
    ];

    i2c.blocking_write(MAX_M10S_ADDRESS, &poll_pvt).unwrap();

    // Initialize the parser
    let mut parser: Parser<ublox::FixedBuffer<256>, Proto31> = Parser::new_fixed();
    // let mut parser = Parser::default

    let mut buf: [u8; 256] = [0; 256];

    loop {
        i2c.blocking_write(MAX_M10S_ADDRESS, &poll_pvt).unwrap();
        // Timer::after_millis(50).await;

        i2c.blocking_read(MAX_M10S_ADDRESS, &mut buf).unwrap();

        let mut it = parser.consume_ubx(&buf);
        loop {
            match it.next() {
                Some(Ok(UbxPacket::Proto31(p))) => {
                    info!("proto31 packet received");
                    handle_packet_proto31(p);
                }

                Some(Err(e)) => {
                    info!("PARSER ERROR - Received malformed packet");
                }
                None => {
                    info!("No more packets to parse");
                    // The internal buffer is now empty
                    break;
                }
            }
        }

        Timer::after_millis(800).await;
    }

    // let mut buffer: [u8; 128] = [0; 128];
    // i2c.blocking_read(MAX_M10S_ADDRESS, &mut buffer).unwrap();

    // In a real application, replace this with your UART reading logic
    // let mut uart = board.uart1;
    // uart.blocking_read(&mut buffer).unwrap();

    // info!("here is the buffer: {:X}", buffer);
    // Consume bytes and iterate over packets
    // let mut it = parser.consume_ubx(&buffer);
    // loop {
    //     match it.next() {
    //         Some(Ok(UbxPacket::Proto31(p))) => {
    //             info!("proto31 packet received");
    //             handle_packet_proto31(p);
    //         }

    //         Some(Err(e)) => {
    //             info!("PARSER ERROR - Received malformed packet");
    //         }
    //         None => {
    //             info!("No more packets to parse");
    //             // The internal buffer is now empty
    //             break;
    //         }
    //     }
    // }

    loop {}
}

fn handle_packet_proto31(p: ublox::proto31::PacketRef) {
    info!("Received UBX packet");
    match p {
        ublox::proto31::PacketRef::NavPvt(nav_pvt) => {
            info!("Speed: {} [m/s]", nav_pvt.ground_speed_2d());
            info!("Latitude: {} [degrees]", nav_pvt.latitude());
            info!("Longitude: {} [degrees]", nav_pvt.longitude());
            info!("Height: {} [m]", nav_pvt.height_msl());
            info!("Satellites: {}", nav_pvt.num_satellites());
        }
        ublox::proto31::PacketRef::EsfMeas(esf_meas) => {
            for data in esf_meas.data() {
                info!("ESF MEAS DATA");
            }
        }
        _ => (),
    }
}
