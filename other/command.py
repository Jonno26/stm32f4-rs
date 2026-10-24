import struct
import time

import serial

MAGIC = 0xA5
ACK_COMMAND = 0x80
NACK_COMMAND = 0x81
MAX_PAYLOAD = 256

# 0x7E is currently an unknown command in the firmware.
DUMMY_COMMAND = 0x0
DUMMY_PAYLOAD = b"\x10\x20\x30"


def read_exact(port: serial.Serial, size: int, timeout: float) -> bytes:
    """Read exactly size bytes, raising TimeoutError if the deadline expires."""
    result = bytearray()
    deadline = time.monotonic() + timeout

    while len(result) < size:
        if time.monotonic() >= deadline:
            raise TimeoutError(
                f"Timed out reading {size} bytes; received {len(result)}"
            )

        chunk = port.read(size - len(result))
        if chunk:
            result.extend(chunk)

    return bytes(result)


def write_all(port: serial.Serial, data: bytes) -> None:
    """Write all bytes, handling a partial write if one is reported."""
    remaining = memoryview(data)

    while remaining:
        written = port.write(remaining)
        if written == 0:
            raise TimeoutError("Timed out writing to serial port")
        remaining = remaining[written:]


def send_command(
    port: serial.Serial,
    command: int,
    payload: bytes = b"",
    timeout: float = 2.0,
) -> tuple[bool, bytes]:
    """Send one framed command and return (was_acked, response_payload)."""
    if not 0 <= command <= 0xFF:
        raise ValueError("command must fit in one byte")
    if len(payload) > MAX_PAYLOAD:
        raise ValueError(f"payload exceeds {MAX_PAYLOAD} bytes")

    # Frame: [magic][payload length: u16 little-endian][command][payload]
    frame = (
        bytes([MAGIC])
        + struct.pack(">H", len(payload))
        + bytes([command])
        + payload
    )

    write_all(port, frame)
    port.flush()

    # Response frame has the same four-byte header.
    header = read_exact(port, 4, timeout)
    magic, response_length, response_command = struct.unpack(">BHB", header)

    if magic != MAGIC:
        raise ValueError(f"Bad response magic: 0x{magic:02X}")
    if response_length > MAX_PAYLOAD:
        raise ValueError(f"Response payload too large: {response_length}")

    response_payload = read_exact(port, response_length, timeout)

    if response_command == ACK_COMMAND:
        return True, response_payload
    if response_command == NACK_COMMAND:
        return False, response_payload

    raise ValueError(f"Unexpected response command: 0x{response_command:02X}")


if __name__ == "__main__":
    port_name = "COM4"  # Change this to your board's COM port.

    # CDC ACM generally ignores baudrate, but pyserial requires a value.
    # A short read timeout lets read_exact enforce its own overall deadline.
    with serial.Serial(port_name, baudrate=115200, timeout=0.1, write_timeout=2) as port:
        acked, response_payload = send_command(
            port,
            DUMMY_COMMAND,
            DUMMY_PAYLOAD,
        )

        if acked:
            print(f"ACK; response payload: {response_payload.hex()}")
        else:
            print(f"NACK; response payload: {response_payload.hex()}")