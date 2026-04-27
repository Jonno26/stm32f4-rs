#![no_std]
#![no_main]

use embassy_time::{Duration, block_for};
pub use stm32f4_rs::prelude::*;

use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut board = STM32F4::init();

    let m10_reset = board.m10_reset;
    info!("m10 reset pin is: {}", m10_reset.get_output_level());

    let mut i2c = board.i2c1;
    i2c = helpers::i2c_scanner(i2c);

    // UBX NAV-PVT poll request format
    // let poll_pvt: [u8; 8] = [
    //     0xB5,
    //     0x62, // sync chars
    //     NavPvt::CLASS,
    //     0x07, // class = NAV, id = PVT
    //     0x00,
    //     0x00, // payload length = 0
    //     0x08,
    //     0x19, // checksum
    // ];

    // Initialize the parser
    // let mut parser: Parser<ublox::FixedBuffer<256>, Proto33> = Parser::new_fixed();
    // let mut parser = ParserBuilder::new()
    //     .with_protocol::<Proto33>()
    //     .with_fixed_buffer::<1024>();

    // let mut r_buf: [u8; 2] = [0; 2];
    // let write_buf = [MAX_M10S_NUM_BYTES_HIGH_ADDR];
    // let mut buf: [u8; 256] = [0; 256];

    let mut max_m10s = MaxM10s::new(i2c);

    max_m10s.get_version();
    // max_m10s.get_pvt();

    loop {
        // max_m10s.get_pvt();
        // block_for(Duration::from_millis(1000));
    }

    // loop {
    //     i2c.blocking_write(MAX_M10S_ADDRESS, &poll_pvt).unwrap();

    //     // // Read out how many bytes are available to read
    //     // i2c.blocking_write_read(MAX_M10S_ADDRESS, &write_buf, &mut r_buf)
    //     //     .unwrap();

    //     // let bytes_available = ((r_buf[0] as usize) << 8) | (r_buf[1] as usize);

    //     // if bytes_available > 1024 {
    //     // info!("Bytes available to read: {}", bytes_available);

    //     i2c.blocking_read(MAX_M10S_ADDRESS, &mut buf).unwrap();

    //     let mut it = parser.consume_ubx(&buf);
    //     loop {
    //         match it.next() {
    //             Some(Ok(UbxPacket::Proto33(p))) => {
    //                 info!("proto33 packet received");
    //                 handle_packet_proto33(p);
    //             }

    //             Some(Err(e)) => match e {
    //                 ParserError::InvalidChecksum { expect, got } => {
    //                     error!(
    //                         "PARSER ERROR - INVALID CHECKSUM - expected {:02X} but got {:02X}",
    //                         expect, got
    //                     );
    //                 }
    //                 ParserError::InvalidField { packet, field } => {
    //                     error!(
    //                         "PARSER ERROR - INVALID FIELD - packet: {}, field: {}",
    //                         packet, field
    //                     );
    //                 }
    //                 ParserError::InvalidPacketLen {
    //                     packet,
    //                     expect,
    //                     got,
    //                 } => {
    //                     error!(
    //                         "PARSER ERROR - INVALID PACKET LENGTH- packet: {}, expected: {}, got: {}",
    //                         packet, expect, got
    //                     );
    //                 }
    //                 ParserError::OutOfMemory { required_size } => {
    //                     error!(
    //                         "PARSER ERROR - Parser Buffer Too Small - required size: {}",
    //                         required_size
    //                     );
    //                 }
    //             },
    //             None => {
    //                 // debug!("No more packets to parse - buffer cannot yield another full packet");
    //                 // The internal buffer is now empty
    //                 break;
    //             }
    //         }
    //     }
    // }

    // }
}
