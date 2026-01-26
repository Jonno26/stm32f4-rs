use crate::prelude::*;

pub const IMU_ADDR: u8 = 0x68;
pub const IMU_WHOAMI_ADDR: [u8; 1] = [0x75];

pub struct IMU<'a> {
    i2c: I2c<'a, Blocking, Master>,
}

impl<'a> IMU<'a> {
    pub fn new(i2c: I2c<'a, Blocking, Master>) -> Self {
        Self { i2c }
    }

    pub fn init(&mut self) -> BoardResult<()> {
        let mut buf: [u8; 1] = [0; 1];
        let res = self
            .i2c
            .blocking_write_read(IMU_ADDR, &IMU_WHOAMI_ADDR, &mut buf)?;

        defmt::assert_eq!(IMU_ADDR, buf[0]);

        Ok(res)
    }
}
