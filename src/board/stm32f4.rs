use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    Config, Peri, PeripheralType, Peripherals,
    gpio::{AnyPin, Level, Output, Speed},
    i2c::{Config as I2cConfig, I2c, Master, SclPin, SdaPin},
    peripherals::{I2C2, USB_OTG_FS},
    rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllQDiv,
        PllSource, Sysclk, mux,
    },
    spi::{Config as SpiConfig, Spi},
    time::Hertz,
    usart::{Config as UsartConfig, Uart},
};

use embassy_stm32::peripherals::{I2C1, PA4, PA5, PA6, PA7, PA8, PA11, PA12, PB6, PB7, SPI1};

use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::prelude::*;

pub struct STM32F4<'a> {
    pub red_led: Output<'a>,
    pub pa0: Peri<'a, AnyPin>,
    pub pb9: Peri<'a, AnyPin>,
    // pub spi1: SpiPins<SPI1, PA7, PA6, PA5, PA4>,
    // pub cs: Peri<'static, AnyPin>,
    pub pa11: Peri<'a, PA11>,
    pub pa12: Peri<'a, PA12>,

    pub m10_reset: Output<'a>,

    pub i2c1: I2c<'a, Blocking, Master>,

    pub uart1: Uart<'a, Blocking>,

    pub usb_otg_fs: Peri<'a, USB_OTG_FS>,
}

impl<'a> STM32F4<'a> {
    pub fn init() -> Self {
        let stm_config = STM32F4::init_clocks();

        let p = embassy_stm32::init(stm_config);
        let mut i2c_config = I2cConfig::default();
        i2c_config.frequency = Hertz::khz(400);
        i2c_config.timeout = embassy_time::Duration::from_millis(2000);
        i2c_config.scl_pullup = false;
        i2c_config.sda_pullup = false;
        // i2c_config.gpio_speed = Speed::High;

        let i2c1 = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, i2c_config);

        let red_led = Output::new(p.PB13, Level::Low, Speed::Low);
        let m10_reset = Output::new(p.PA8, Level::High, Speed::Low);

        let mut uart_config = UsartConfig::default();
        uart_config.baudrate = 9600;
        uart_config.parity = embassy_stm32::usart::Parity::ParityNone;
        uart_config.stop_bits = embassy_stm32::usart::StopBits::STOP1;
        uart_config.data_bits = embassy_stm32::usart::DataBits::DataBits8;
        let uart1 = Uart::new_blocking(p.USART1, p.PA10, p.PA9, uart_config).unwrap();

        Self {
            red_led,
            pa0: p.PA0.into(),
            pb9: p.PB9.into(),
            pa11: p.PA11,
            pa12: p.PA12,
            m10_reset,
            i2c1, // cs: p.PA4.into(),
            uart1,
            usb_otg_fs: p.USB_OTG_FS,
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

        // stm_config.rcc.pll_src = PllSource::HSE;
        // stm_config.rcc.pll = Some(Pll {
        //     prediv: PllPreDiv::DIV12,
        //     mul: PllMul::MUL168,
        //     divp: Some(PllPDiv::DIV4), // (24MHz/12) * 168 / 4 = 84MHz
        //     divq: Some(PllQDiv::DIV7), // (24MHz/12) * 168 / 7 = 48MHz. USB clock needs to be 48MHz
        //     divr: None,
        // });

        // stm_config.rcc.ahb_pre = AHBPrescaler::DIV1;
        // stm_config.rcc.apb1_pre = APBPrescaler::DIV4;
        // stm_config.rcc.apb2_pre = APBPrescaler::DIV2;
        // stm_config.rcc.sys = Sysclk::PLL1_P;
        // stm_config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;

        stm_config
    }
}
