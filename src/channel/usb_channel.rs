pub use crate::prelude::*;

pub use embassy_usb::class::cdc_acm::CdcAcmClass;
pub use embassy_usb::driver::Driver;

const USB_PACKET_SIZE: usize = 64;

pub struct Usb<'d, D: embassy_usb::driver::Driver<'d>> {
    class: embassy_usb::class::cdc_acm::CdcAcmClass<'d, D>,
    rx: [u8; USB_PACKET_SIZE],
    rx_pos: usize,
    rx_len: usize,
}

impl<'d, D: embassy_usb::driver::Driver<'d>> Usb<'d, D> {
    pub fn new(class: embassy_usb::class::cdc_acm::CdcAcmClass<'d, D>) -> Self {
        Self {
            class,
            rx: [0; USB_PACKET_SIZE],
            rx_pos: 0,
            rx_len: 0,
        }
    }

    pub async fn wait_connection(&mut self) {
        self.class.wait_connection().await;
    }
}

impl<'d, D: embassy_usb::driver::Driver<'d>> Channel for Usb<'d, D> {
    async fn read(&mut self, out: &mut [u8]) -> Result<usize, ChannelError> {
        if out.is_empty() {
            return Ok(0);
        }

        // Fetch a USB packet only when the previous one has been consumed.
        while self.rx_pos == self.rx_len {
            self.rx_len = self.class.read_packet(&mut self.rx).await?;
            self.rx_pos = 0;
            // A zero-length USB packet contains no stream bytes; keep waiting.
        }

        let available = self.rx_len - self.rx_pos;
        let count = core::cmp::min(out.len(), available);

        out[..count].copy_from_slice(&self.rx[self.rx_pos..self.rx_pos + count]);
        self.rx_pos += count;

        Ok(count)
    }

    async fn write_all(&mut self, bytes: &[u8]) -> Result<(), ChannelError> {
        let original_len = bytes.len();
        let mut remaining = bytes;

        while remaining.len() >= USB_PACKET_SIZE {
            self.class
                .write_packet(&remaining[..USB_PACKET_SIZE])
                .await?;
            remaining = &remaining[USB_PACKET_SIZE..];
        }

        if !remaining.is_empty() {
            self.class.write_packet(remaining).await?;
        } else if original_len != 0 && original_len % USB_PACKET_SIZE == 0 {
            // Terminate an exactly full-size USB transfer for host serial readers.
            self.class.write_packet(&[]).await?;
        }

        Ok(())
    }
}
