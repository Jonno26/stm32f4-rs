#![no_std]
#![no_main]

use defmt::*;
use mpu6050::*;

use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, Speed},
};

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;
pub use stm32f4_rs::prelude::*;

use {defmt_rtt as _, panic_probe as _};

static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, Async, Master>>> = StaticCell::new();

#[embassy_executor::main]
async fn spawner(_spawner: Spawner) {
    let x = main().await;
    match x {
        Ok(_) => {}
        Err(e) => match e {
            BoardError::SpiError => error!("{}", e),
            BoardError::Other => error!("{}", e),
            BoardError::I2CError(bus_error) => {
                error!("{}", e);
                let bus_error_kind = bus_error.kind();
                match bus_error_kind {
                    embedded_hal::i2c::ErrorKind::ArbitrationLoss => error!("i2c arbitration loss"),
                    embedded_hal::i2c::ErrorKind::NoAcknowledge(_) => error!("i2c nack"),
                    embedded_hal::i2c::ErrorKind::Other => error!("other i2c error"),
                    embedded_hal::i2c::ErrorKind::Overrun => error!("i2c overrun error"),
                    embedded_hal::i2c::ErrorKind::Bus => error!("i2c bus error"),
                    _ => {}
                }
            }
        },
    }
}

async fn main() -> BoardResult<(), I2cDeviceError<i2c::Error>> {
    let mut board = STM32F4::init();
    info!("Hello World!");
    let internal_i2c = board.internal_i2c;
    // let mut mpu = mpu6050::Mpu6050::new(i2c);
    // let mut delay = Delay;
    // let _ = mpu.init(&mut delay).unwrap();

    // let i2c = helpers::i2c_scanner_blocking(i2c);
    let internal_i2c = helpers::i2c_scanner(internal_i2c).await;
    info!("after i2c bus scan");
    let i2c_bus = I2C_BUS.init(Mutex::new(internal_i2c));
    // let gps_i2c = I2cDevice::new(i2c_bus);
    let temp_i2c = I2cDevice::new(i2c_bus);

    // let imu_int = ExtiInput::new(
    //     board.imu_int_pin,
    //     board.imu_int_ch,
    //     Pull::None,
    // );

    // info!("imu_int level is: {}", imu_int.get_level());

    // let (pin, exti) = imu_int.free();

    // let imu_input = Input::new(board.imu_int_pin, Pull::None);
    // info!("imu_input pin level is: {}", imu_input.level());

    let mut temp_sensor = TempSensor::new(temp_i2c);
    // let (temp, rh) = temp_sensor.read_temperature_and_humidity().await?;
    // info!("temp is: {}°C, rh is: {}%", temp, rh);

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
