#![no_std]
#![no_main]

use embassy_time::{Duration, Timer};
pub use stm32f4_rs::prelude::*;

use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, Async, Master>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = STM32F4::init();

    let m10_reset = board.m10_reset;
    info!("m10 reset pin is: {}", m10_reset.get_output_level());

    let i2c = board.temp_i2c;
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));
    let gps_i2c = I2cDevice::new(i2c_bus);
    let temp_i2c = I2cDevice::new(i2c_bus);

    // i2c = helpers::i2c_scanner(i2c);

    // UBX NAV-PVT poll request format
    // let poll_pvt: [u8; 8] = [
    //     0xB5,
    //     0x62, // sync chars
    //     MonVer::CLASS,
    //     MonVer::ID, // class = NAV, id = PVT
    //     0x00,
    //     0x00, // payload length = 0
    //     0x0E,
    //     0x34, // checksum
    // ];

    // Initialize the parser
    // let mut parser: Parser<ublox::FixedBuffer<2048>, Proto33> = Parser::new_fixed();
    // let mut parser = ParserBuilder::new()
    //     .with_protocol::<Proto33>()
    //     .with_fixed_buffer::<2048>();

    // let mut r_buf: [u8; 2] = [0; 2];
    // let write_buf = [MAX_M10S_NUM_BYTES_HIGH_ADDR];
    // let mut buf: [u8; 256] = [0; 256];

    let mut max_m10s = MaxM10s::new(gps_i2c);

    // max_m10s.get_version();
    // max_m10s.get_sec_sig_msg();

    loop {
        max_m10s.get_pvt().await.unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
