#![no_std]
#![no_main]

use core::str;
use defmt::*;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_time::{Delay, Timer};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use {defmt_rtt as _, panic_probe as _};

pub use stm32f4_rs::prelude::*;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::main]
async fn spawner(_spawner: Spawner) {
    let x = main().await;
    match x {
        Ok(_) => {}
        Err(e) => error!("Some error occured: {:?}", e),
    }
}

async fn main() -> BoardResult<(), embassy_stm32::i2c::Error> {
    let mut board = STM32F4::init();
    info!("Hello World!");

    // let i2c = board.i2c1;

    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb::Config::default();

    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = false;

    let driver = Driver::new_fs(
        board.usb_otg_fs,
        board.pa12,
        board.pa11,
        Irqs,
        &mut ep_out_buffer,
        config,
    );

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    let mut channel = Usb::new(class);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // // Do stuff with the class!
    let echo_fut = async {
        loop {
            channel.wait_connection().await;
            info!("Connected");
            // let _ = echo(&mut class).await;
            let _ = wait_for_command(&mut channel).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;

    // loop {
    //     board.red_led.toggle();
    //     Timer::after_millis(500).await;
    // }

    Ok(())
}

async fn wait_for_command(
    channel: &mut impl Channel,
) -> BoardResult<(), I2cDeviceError<i2c::Error>> {
    let mut command = 0;
    let mut payload = [0u8; MAX_PAYLOAD];

    let payload_len = read_frame(channel, &mut command, &mut payload).await?;

    let x = Command::convert_from(command);
    if x == Command::UnknownCommand {
        send_nack(channel).await?;
    } else {
        // received a valid command
        send_ack(channel).await?;
    }

    Ok(())
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => defmt::panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    // let data: [u8; 2] = [2, 4];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        match str::from_utf8(&data) {
            Ok(s) => info!("Successfully converted: {}", s),
            Err(_) => error!("Failed to convert from bytes to UTF-8"),
        }
        // Timer::after_millis(50).await;
        class.write_packet(&data).await?;
        info!("echo of length: {}", n);
    }
}
