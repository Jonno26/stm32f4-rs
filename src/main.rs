#![no_std]
#![no_main]

use defmt::*;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};

use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
// pub mod board;
// pub use board::*;
pub use stm32f4_rs::prelude::*;

#[embassy_executor::main]
async fn spawner(_spawner: Spawner) {
    let x = main().await;
    match x {
        Ok(_) => {}
        Err(e) => error!("Some error occured: {:?}", e),
    }
}

async fn main() -> BoardResult<()> {
    let stm32f4_board = STM32F4::init();
    info!("Hello World!");

    let mut led = Output::new(stm32f4_board.led_pin, Level::High, Speed::High);

    // let spi = Spi::new_blocking(
    //     stm32f4_board.spi1.spi,
    //     stm32f4_board.spi1.sck,
    //     stm32f4_board.spi1.mosi,
    //     stm32f4_board.spi1.miso,
    //     stm32f4_board.spi1.config,
    // );

    let i2c = stm32f4_board.i2c1;

    // let mut read_buf: [u8; 2] = [0; 2];

    let mut temp_sensor = TempSensorHAL::new(i2c);

    let (temp, rh) = temp_sensor.read_temperature_and_humidity()?;
    info!("temp is: {}°C, rh is: {}%", temp, rh);

    let i2c = temp_sensor.inner();

    let mut imu = IMU::new(i2c);
    imu.init()?;

    loop {
        // info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        // info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
