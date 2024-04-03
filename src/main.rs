
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};

use serialport5;
use serialport5::SerialPort;

use std::io::{Read, Write};

// TODO: consider converting to: https://rust-cli-recommendations.sunshowers.io/handling-arguments.html

/// A feature-rich UART serial terminal, written in Rust
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Opens the serial port for communication
    #[clap(alias("o"))]
    OpenPort {
        /// Baud rate for serial communication
        #[arg(short = 'b', long, default_value_t = 115200)]
        baud_rate: u32,

        /// Serial port to use for communication
        #[arg(short = 'p', long)]
        serial_port: String,
    },

    /// List available serial ports, then exit
    #[clap(alias("l"), alias("ls"))]
    ListPorts,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Some(Commands::ListPorts) => {
            list_ports();
        }
        Some(Commands::OpenPort { baud_rate, serial_port }) => {
            open_serial_port_and_read(baud_rate, serial_port);
        }
        None => {
            println!("No command provided. One day, this will prompt for settings...");
        }
    }
}

fn list_ports() {
    let ports = serialport5::available_ports().expect("No serial ports found!");

    if ports.is_empty() {
        println!("No serial ports found!");
        return;
    }

    println!("Available serial ports ({} ports):", ports.len());
    for (index, port) in ports.iter().enumerate() {
        println!("--- Port #{}: {}", index, port.port_name);
    }
}

fn open_serial_port_and_read(baud_rate: u32, serial_port: String) {

    let mut port = SerialPort::builder()
        .baud_rate(baud_rate)
        .read_timeout(Some(Duration::from_millis(10)))
        .open(serial_port.clone())
        .expect("Failed to open port"); // FIXME

    println!("Opened serial port '{}' at {} baud.", serial_port, baud_rate);

    // Flush the serial port buffer
    match port.flush() {
        Ok(_) => { }
        Err(e) => {
            eprintln!("Error flushing serial port buffer: {}", e);
        }
    }

    // Read from the serial port continuously
    loop {
        let mut serial_buf: Vec<u8> = vec![0; 32];
        let val = port.read(serial_buf.as_mut_slice());

        match val {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    let data = &serial_buf[..bytes_read];
                    let data_str = std::str::from_utf8(data).unwrap();
                    print!("{}", data_str);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Do nothing
                // TODO: consider printing an "end-of-stream" character here, marking the next one as the start of a new message (maybe with timestamp), etc.
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {}", e);
                break;
            }
        }
    }
}
