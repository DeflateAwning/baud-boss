pub enum CurrentScreen {
    PickSerialPort,
    PickBaudRate,
    Main,
    Config1,
    Help,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub app_config: AppConfig,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::PickSerialPort,
            app_config: AppConfig {
                serial_port: String::new(),
                baud_rate: 115200,
                end_of_line: String::from("\n"),
                data_bits: 8,
                parity: serialport5::Parity::None,
                stop_bits: serialport5::StopBits::One,
            },
        }
    }
    
}

pub struct AppConfig {
    pub serial_port: String,
    pub baud_rate: u32,
    
    pub end_of_line: String,

    // 8N1 parameters // TODO: do something with these
    pub data_bits: u8,
    pub parity: serialport5::Parity,
    pub stop_bits: serialport5::StopBits,
}

