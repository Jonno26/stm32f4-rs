#![no_std]
#![no_main]

use defmt::*;
use embedded_hal::delay::DelayNs;
use mpu6050::*;

use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, Speed},
};

use embassy_time::{Delay, Timer};
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
    let mut board = STM32F4::init();
    info!("Hello World!");
    let i2c = board.temp_i2c;
    // let mut mpu = mpu6050::Mpu6050::new(i2c);
    // let mut delay = Delay;
    // let _ = mpu.init(&mut delay).unwrap();

    // let i2c = helpers::i2c_scanner(i2c);

    // let imu_int = ExtiInput::new(
    //     board.imu_int_pin,
    //     board.imu_int_ch,
    //     Pull::None,
    // );

    // info!("imu_int level is: {}", imu_int.get_level());

    // let (pin, exti) = imu_int.free();

    let imu_input = Input::new(board.imu_int_pin, Pull::None);
    info!("imu_input pin level is: {}", imu_input.get_level());

    let imu_input_pin = imu_input.free();

    let mut temp_sensor = TempSensor::new(i2c);
    let (temp, rh) = temp_sensor.read_temperature_and_humidity().await?;
    info!("temp is: {}°C, rh is: {}%", temp, rh);

    // let i2c = temp_sensor.inner();

    loop {
        // get roll and pitch estimate
        // let acc = mpu.get_acc_angles().unwrap();
        // for row in &acc {
        //     info!("row is: {}", row);
        // }

        // get gyro data, scaled with sensitivity
        // let gyro = mpu.get_gyro().unwrap();
        // for row in &gyro {
        //     info!("row is: {}", row);
        // }

        // // get accelerometer data, scaled with sensitivity
        // // let acc = mpu.get_acc().unwrap();
        // // for row in &acc {
        // //     info!("row is: {}", row);
        // // }
        // info!("--------------");

        // get imu temp
        // let temp = mpu.get_temp().unwrap();
        // println!("temp: {:?}c", temp);

        let (temp, rh) = temp_sensor.read_temperature_and_humidity().await?;
        info!("temp is: {}°C, rh is: {}%", temp, rh);

        board.red_led.toggle();
        Timer::after_millis(500).await;
    }
}
