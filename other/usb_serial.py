import serial 
import time


port = 'COM3'
baudrate = 115200 # doesn't matter for native usd but this must be defined


try: 
    ser = serial.Serial(port, baudrate, timeout=1)
    print(f"Port {port} opened successfully at {baudrate} baud")
    # time.sleep(2)  # Wait for device to initialize

    message = b'Hello STM32 this is Jonno26 yesshddddddddddddddd yayayayaya ahahahahanig\n'
    print(f"Sending: {message.decode('utf-8').rstrip()}")
    ser.write(message)
    ser.flush()  # Ensure data is sent immediately
    
    # Wait a bit for device to process and respond
    # time.sleep(0.1)s
    
    # Read response - use readline() with timeout to wait for a complete line
    # The timeout=2 set above ensures we wait up to 2 seconds for data
    try:
        response_data = ser.readline()
        if response_data:
            print(f"Data received on first attempt: {len(response_data)} bytes")
        else:
            print("Timeout waiting for response")
    except Exception as read_error:
        print(f"Error reading: {read_error}")
        response_data = b''
    
    if response_data:
        try:
            data = response_data.decode('utf-8').rstrip()
            print(f"Received: {data}")
        except UnicodeDecodeError:
            print(f"Received (raw bytes): {response_data}")
    else:
        print("No response received from device")
    
    ser.close()

except serial.SerialException as e:
    print(f"Serial Port Error: {e}")
except Exception as e:
    print(f"Error: {e}")


