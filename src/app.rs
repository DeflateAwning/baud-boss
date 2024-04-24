use ratatui::widgets::{ListState, ScrollbarState};
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

    pub main_input: String, // TODO: maybe make this a Vec<u8> instead, probably
    pub main_input_send_history: Vec<String>, // for up-arrow history
    pub main_input_send_history_index: Option<usize>,
    pub main_input_typing_in_progress_but_not_sent: Option<String>, // so that if you look through the send history, you can still send the current in-progress message
    pub main_input_cursor_position: Option<usize>,
    pub main_screen_transfer_log: Vec<VisibleTransferData>,

    pub bound_serial_port: Option<Box<serialport5::SerialPort>>,

    pub main_screen_active_region: MainScreenActiveRegion,
    pub main_screen_vert_scroll_state: ScrollbarState,
    pub main_screen_horiz_scroll_state: ScrollbarState,
    pub main_screen_vert_scroll_val: usize,
    pub main_screen_horiz_scroll_val: usize,
    pub main_screen_vert_scroll_pos: ScrollPosition,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::PickSerialPort,
            app_config: AppConfig::default(),

            pick_serial_port_list_state: ListStateTracker::default(),
            selected_serial_port: None,
            general_error_message: None,
            
            pick_baud_rate_input_field: String::new(),

            main_input: String::new(),
            main_input_send_history: Vec::new(),
            main_input_send_history_index: None,
            main_input_typing_in_progress_but_not_sent: None,
            main_input_cursor_position: None,
            main_screen_transfer_log: Vec::new(),

            bound_serial_port: None,

            main_screen_active_region: MainScreenActiveRegion::Input,
            main_screen_vert_scroll_state: ScrollbarState::default(),
            main_screen_horiz_scroll_state: ScrollbarState::default(),
            main_screen_vert_scroll_val: 0,
            main_screen_horiz_scroll_val: 0,
            main_screen_vert_scroll_pos: ScrollPosition::PinnedAtEnd,
        }
    }
    
    pub fn add_rxd_serial_data_to_transfer_log(&mut self, new_data: Vec<u8>) {
        self.main_screen_transfer_log.push(
            VisibleTransferData::SerialData(
                // FIXME: improve support for non-UTF-8 data
                String::from_utf8(new_data).expect("Incoming serial data should be UTF-8, for now")
            )
        );
    }

    pub fn add_echo_to_transfer_log(&mut self, new_data: Vec<u8>) {
        self.main_screen_transfer_log.push(
            VisibleTransferData::EchoData(
                // TODO: remove clone one we're done with debugging
                String::from_utf8(new_data.clone()).expect("Data to send should be UTF-8, for now")
            )
        );
    }

    pub fn add_error_to_transfer_log(&mut self, new_data: String) {
        self.main_screen_transfer_log.push(
            VisibleTransferData::ErrorData(new_data)
        );
    }
}

pub enum VisibleTransferData {
    SerialData(String),
    EchoData(String),
    ErrorData(String),
    // TODO: maybe other
}

#[derive(Debug, PartialEq)]
pub enum ScrollPosition {
    FinitePosition,
    PinnedAtEnd,
}

pub struct AppConfig {
    // NOTE: serial_port is not here because it's not cross-environment; it will however be a CLI argument
    
    pub baud_rate: Option<u32>, // baud_rate is optional because it's selected in the UI
    
    pub end_of_line: String,

    // 8N1 parameters // FIXME: do something with these; they're not used yet in the init
    pub data_bits: u8,
    pub parity: serialport5::Parity,
    pub stop_bits: serialport5::StopBits,



    // pub line_wrap: bool, // TODO: implement line wrap
    // pub show_borders: bool, // TODO: implement show/hide borders
    // pub show_help: bool, // TODO: implement show/hide help at bottom
    pub echo_mode: EchoMode,

}

impl AppConfig {
    pub fn default() -> Self {
        Self {
            baud_rate: None,
            end_of_line: String::from("\n"),
            data_bits: 8,
            parity: serialport5::Parity::None,
            stop_bits: serialport5::StopBits::One,

            echo_mode: EchoMode::On,
        }
    }
}

pub enum MainScreenActiveRegion {
    Input,
    InputEolChoice, // PERMANENTLY DISABLED, for now
    OutputScrollBars,
}

impl MainScreenActiveRegion {
    pub fn next(&self) -> MainScreenActiveRegion {
        match self {
            MainScreenActiveRegion::Input => MainScreenActiveRegion::OutputScrollBars,
            MainScreenActiveRegion::OutputScrollBars => MainScreenActiveRegion::Input,
            MainScreenActiveRegion::InputEolChoice => unimplemented!(),
        }
    }

    pub fn prev(&self) -> MainScreenActiveRegion {
        match self {
            MainScreenActiveRegion::Input => MainScreenActiveRegion::OutputScrollBars,
            MainScreenActiveRegion::OutputScrollBars => MainScreenActiveRegion::Input,
            MainScreenActiveRegion::InputEolChoice => unimplemented!(),
        }
    }
}

pub enum EchoMode {
    On,
    Off,
}

