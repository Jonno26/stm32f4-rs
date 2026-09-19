use defmt::debug;
use embedded_hal_async::i2c::I2c as I2cAsync;

pub const TEMP_ADDR: u8 = 0x44;
pub const TEMP_STATUS_COMMAND: [u8; 2] = [0xF3, 0x2D];
/// Single shot, high repeatabiltiy with clock stretching enabled
pub const TEMP_MEAS_COMMAND: [u8; 2] = [0x2C, 0x06];
pub const TEMP_SOFT_RESET: [u8; 2] = [0x30, 0xA2];
pub const DIVISOR: f32 = 65535.0;

///## SHT30-DIS-B2.5KS Temp Sensor
pub struct TempSensor<I2C> {
    i2c: I2C,
}

impl<I2C> TempSensor<I2C>
where
    I2C: I2cAsync,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }
    pub async fn read_status_register(&mut self, read_buf: &mut [u8]) -> Result<(), I2C::Error> {
        self.i2c
            .write_read(TEMP_ADDR, &TEMP_STATUS_COMMAND, read_buf)
            .await
    }

    pub async fn read_temperature(&mut self) -> Result<f32, I2C::Error> {
        let mut read_buf: [u8; 2] = [0; 2];

        self.i2c
            .write_read(TEMP_ADDR, &TEMP_MEAS_COMMAND, &mut read_buf)
            .await?;

        let raw_temp: u16 = ((read_buf[0] as u16) << 8) | (read_buf[1] as u16);
        // debug!("raw_temp is: {:b}", raw_temp);

        let temp = Self::convert_raw_temp(raw_temp);

        Ok(temp)
    }

    pub async fn read_temperature_and_humidity(&mut self) -> Result<(f32, f32), I2C::Error> {
        let mut read_buf: [u8; 5] = [0; 5];

        self.i2c
            .write_read(TEMP_ADDR, &TEMP_MEAS_COMMAND, &mut read_buf)
            .await?;

        let raw_temp: u16 = ((read_buf[0] as u16) << 8) | (read_buf[1] as u16);
        // debug!("raw temp is: {:b}", raw_temp);
        let temp = Self::convert_raw_temp(raw_temp);

        let raw_humidity: u16 = ((read_buf[3] as u16) << 8) | (read_buf[4] as u16);
        // debug!("raw humidity is: {:b}", raw_humidity);
        let rh = Self::convert_raw_humidity(raw_humidity);
        Ok((temp, rh))
    }

    pub async fn soft_reset(&mut self) -> Result<(), I2C::Error> {
        debug!("temp sensor soft reset");
        self.i2c.write(TEMP_ADDR, &TEMP_SOFT_RESET).await
    }

    pub fn inner(self) -> I2C {
        self.i2c
    }

    pub fn inner_mut(&mut self) -> &I2C {
        &self.i2c
    }

    /// This function converts the raw 16-bit unsigned temperature value to a
    /// temperature in degrees Celsius.
    fn convert_raw_temp(raw_temp: u16) -> f32 {
        // let divisor: f32 = 65536.0;

        let temp = -45.0 + ((175.0 * (raw_temp as f32)) / DIVISOR);
        // debug!("temp is: {}", temp);
        temp
    }

    /// This function converts the raw 16-bit unsigned humdity value to a
    /// humidity in %RH.
    fn convert_raw_humidity(raw_humidity: u16) -> f32 {
        let rh = ((raw_humidity as f32) / DIVISOR) * 100.0;
        rh
    }
}
