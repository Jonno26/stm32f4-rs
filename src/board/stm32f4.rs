use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    Config, Peri, PeripheralType, Peripherals,
    gpio::{AnyPin, Level, Output, Speed},
    i2c::{Config as I2cConfig, I2c, Master, SclPin, SdaPin},
    peripherals::I2C2,
    rcc::{Hse, HseMode, Sysclk},
    spi::{Config as SpiConfig, Spi},
    time::Hertz,
};

use embassy_stm32::peripherals::{I2C1, PA4, PA5, PA6, PA7, PB6, PB7, SPI1};

use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::prelude::*;

pub struct STM32F4<'a> {
    pub led_pin: Peri<'static, AnyPin>,
    pub pa0: Peri<'static, AnyPin>,
    pub pb9: Peri<'static, AnyPin>,
    // pub spi1: SpiPins<SPI1, PA7, PA6, PA5, PA4>,
    pub i2c1: I2c<'a, Blocking, Master>,
    // pub cs: Peri<'static, AnyPin>,
}

impl<'a> STM32F4<'a> {
    pub fn init() -> Self {
        let stm_config = STM32F4::init_clocks();

        let p = embassy_stm32::init(stm_config);
        let i2c_config = I2cConfig::default();
        // let spi_config = SpiConfig::default();

        let i2c1 = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);

        Self {
            led_pin: p.PB13.into(),
            pa0: p.PA0.into(),
            pb9: p.PB9.into(),
            i2c1, // cs: p.PA4.into(),
        }
    }

    pub fn init_clocks() -> Config {
        let mut stm_config = Config::default();

        // Configure board to use the 24MHz HSE crystal
        // Not using PLL atm
        stm_config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(24),
            mode: HseMode::Oscillator,
        });

        stm_config.rcc.sys = Sysclk::HSE;

        stm_config
    }
}
