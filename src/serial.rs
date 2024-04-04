

use serialport5::SerialPort;
use std::{error::Error, time::Duration};

use std::io::{Read, Write};

/// Binds a serial port, returning access to it.
pub fn bind_serial_port(serial_port: &str, baud_rate: u32) -> Result<Box<SerialPort>, Box<dyn Error>> {
    let mut port = SerialPort::builder()
        .baud_rate(baud_rate)
        .read_timeout(Some(Duration::from_millis(10)))
        .open(&serial_port)?;
    
    // Flush the serial port buffer (TX/MOSI)
    match port.flush() {
        Ok(_) => { }
        Err(_e) => {
            panic!("Error flushing serial port buffer right after binding: {}", _e);
        }
    }

    // Read and discard any data in the serial port buffer (RX/MISO)
    let mut serial_buf: Vec<u8> = vec![0; 2000];
    loop {
        let val = port.read(serial_buf.as_mut_slice());
        match val {
            Ok(_bytes_read) => { }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                break;
            } // no data to read, nice
            Err(_e) => {
                panic!("Error reading from serial port buffer right after binding: {}", _e);
            }
        }
    }
    
    Ok(Box::new(port))
}
