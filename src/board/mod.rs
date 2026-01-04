pub mod stm32f4;

pub use stm32f4::*;

// use embassy_stm32::{
//     Peri, PeripheralType,
//     i2c::{Config as I2cConfig, Instance as I2cInstance, SclPin, SdaPin},
//     spi::{Config as SpiConfig, CsPin, Instance as SpiInstance, MisoPin, MosiPin, SckPin},
// };

// pub struct SpiPins<
//     SPI: SpiInstance,
//     MOSI: MosiPin<SPI>,
//     MISO: MisoPin<SPI>,
//     SCK: SckPin<SPI>,
//     CS: CsPin<SPI>,
// > {
//     pub spi: SPI,
//     pub mosi: MOSI,
//     pub miso: MISO,
//     pub sck: SCK,
//     pub cs: CS,
//     pub spi_config: SpiConfig,
// }

// pub struct I2cPins<I2C: I2cInstance, SCL: SclPin<I2C>, SDA: SdaPin<I2C>> {
//     pub i2c: I2C,
//     pub scl: SCL,
//     pub sda: SDA,
//     pub i2c_config: I2cConfig,
// }
