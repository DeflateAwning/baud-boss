use ratatui::widgets::ListState;
use crate::tui_list_state_tracker::ListStateTracker;

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

    pub pick_serial_port_list_state: ListStateTracker,
    pub selected_serial_port: Option<String>, // not in config as it's emphemeral

    // TODO: pick_baud_rate_active_list options

    pub main_input: String, // TODO: maybe make this a Vec<u8> instead
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

                line_wrap: true,
            },

            pick_serial_port_list_state: ListStateTracker::default(),
            selected_serial_port: None,

            main_input: String::new(),
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

    pub line_wrap: bool,
}

