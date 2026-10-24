pub mod rtt_channel;
pub mod usb_channel;
pub use rtt_channel::*;
pub use usb_channel::*;

pub use crate::prelude::*;
pub use embassy_usb::driver::EndpointError;

pub const MAGIC: u8 = 0xA5;
pub const MAX_PAYLOAD: usize = 256;
const ACK_COMMAND: u8 = 0x80;
const NACK_COMMAND: u8 = 0x81;

// Command Frame Format
// [MAGIC: u8][PAYLOAD_LEN: u16 big-endian][COMMAND: u8][PAYLOAD]

pub trait Channel {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>;
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), ChannelError>;

    async fn read_exact(&mut self, mut out: &mut [u8]) -> Result<(), ChannelError> {
        while !out.is_empty() {
            let n = self.read(out).await?;

            // Channel implementations should not return 0 for a non-empty
            // buffer, but guard against it to avoid slicing incorrectly.
            if n == 0 {
                continue;
            }

            out = &mut out[n..];
        }

        Ok(())
    }
}

pub async fn read_frame<C: Channel>(
    channel: &mut C,
    command: &mut u8,
    payload: &mut [u8; MAX_PAYLOAD],
) -> Result<usize, FrameError> {
    // Header: magic, two-byte payload length, command.
    let mut header = [0u8; 4];
    channel.read_exact(&mut header).await?;

    if header[0] != MAGIC {
        return Err(FrameError::BadMagic);
    }
    info!("received header: {}", header);

    let payload_len = u16::from_be_bytes([header[1], header[2]]) as usize;
    if payload_len > payload.len() {
        return Err(FrameError::PayloadTooLarge);
    }

    *command = header[3];
    let c = Command::convert_from(*command);

    channel.read_exact(&mut payload[..payload_len]).await?;

    info!("received payload: {}", payload[..payload_len]);

    Ok(payload_len)
}

pub async fn send_ack<C: Channel>(channel: &mut C) -> Result<(), ChannelError> {
    // [magic, length low, length high, command]
    channel.write_all(&[MAGIC, 0, 0, ACK_COMMAND]).await
}

pub async fn send_nack<C: Channel>(channel: &mut C) -> Result<(), ChannelError> {
    // [magic, length low, length high, command]
    channel.write_all(&[MAGIC, 0, 0, NACK_COMMAND]).await
}

// async fn serve_one<C: Channel>(channel: &mut C) -> Result<(), FrameError> {
//     let mut command = 0;
//     let mut payload = [0u8; MAX_PAYLOAD];

//     let payload_len = read_frame(channel, &mut command, &mut payload).await?;

//     if validate_command(command, &payload[..payload_len]) {
//         // Send ACK only after command and payload validation succeeds.
//         send_ack(channel).await?;
//     }

//     Ok(())
// }

#[derive(Debug)]
pub enum FrameError {
    Channel(ChannelError),
    BadMagic,
    PayloadTooLarge,
}

#[derive(Debug)]
pub enum ChannelError {
    USB,
    UART,
    RTT,
}

impl From<ChannelError> for FrameError {
    fn from(error: ChannelError) -> Self {
        Self::Channel(error)
    }
}

impl<BUS> From<ChannelError> for BoardError<BUS>
where
    BUS: embedded_hal_async::i2c::Error,
{
    fn from(value: ChannelError) -> Self {
        BoardError::ChannelError
    }
}

impl<BUS> From<FrameError> for BoardError<BUS>
where
    BUS: embedded_hal_async::i2c::Error,
{
    fn from(value: FrameError) -> Self {
        BoardError::ChannelFrameError
    }
}

impl From<EndpointError> for ChannelError {
    fn from(_value: EndpointError) -> Self {
        ChannelError::USB
    }
}

pub enum ChannelResponse {
    Ok,
    Error,
}
