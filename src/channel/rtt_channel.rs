use crate::Channel;
use rtt_target::{DownChannel, UpChannel};

pub use crate::prelude::*;

pub struct Rtt {
    down_channel: rtt_target::DownChannel,
    up_channel: rtt_target::UpChannel,
}

impl Channel for Rtt {
    async fn read(&mut self, out: &mut [u8]) -> Result<usize, ChannelError> {
        if out.is_empty() {
            return Ok(0);
        }

        loop {
            let n = self.down_channel.read(out);
            if n != 0 {
                return Ok(n);
            }

            embassy_time::Timer::after_millis(1).await;
        }
    }

    async fn write_all(&mut self, bytes: &[u8]) -> Result<(), ChannelError> {
        let mut written = 0;

        while written < bytes.len() {
            let n = self.up_channel.write(&bytes[written..]);

            if n == 0 {
                // The RTT up-channel buffer has no space yet.
                embassy_time::Timer::after_millis(1).await;
            } else {
                written += n;
            }
        }

        Ok(())
    }
}
