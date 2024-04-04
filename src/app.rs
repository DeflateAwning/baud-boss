use ratatui::widgets::ListState;
use crate::tui_list_state_tracker::ListStateTracker;

// TODO: rename to AppScreen
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
    pub general_error_message: Option<String>,

    pub pick_baud_rate_input_field: String,

    // TODO: pick_baud_rate_active_list options

    pub main_input: String, // TODO: maybe make this a Vec<u8> instead
    pub main_incoming_serial_data: String,

    pub bound_serial_port: Option<Box<serialport5::SerialPort>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::PickSerialPort,
            app_config: AppConfig {
                baud_rate: None,
                end_of_line: String::from("\n"),
                data_bits: 8,
                parity: serialport5::Parity::None,
                stop_bits: serialport5::StopBits::One,

            },

            pick_serial_port_list_state: ListStateTracker::default(),
            selected_serial_port: None,
            general_error_message: None,
            
            pick_baud_rate_input_field: String::new(),

            main_input: String::new(),
            main_incoming_serial_data: String::new(),

            bound_serial_port: None,
        }
    }
    
}

pub struct AppConfig {
    // NOTE: serial_port is not here because it's not cross-environment; it will however be a CLI argument
    
    pub baud_rate: Option<u32>, // baud_rate is optional because it's selected in the UI
    
    pub end_of_line: String,

    // 8N1 parameters // TODO: do something with these
    pub data_bits: u8,
    pub parity: serialport5::Parity,
    pub stop_bits: serialport5::StopBits,

    // pub line_wrap: bool, // TODO: implement line wrap
    // pub show_borders: bool, // TODO: implement show/hide borders
    // pub show_help: bool, // TODO: implement show/hide help at bottom
}

