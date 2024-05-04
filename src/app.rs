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
    pub main_screen_transfer_log: Vec<TransferLogEntry>,

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
        // If the last element in the transfer log is a SerialData. 
        // Otherwise, add a new SerialData element.

        // split by newline, loop through, add each to the transfer log (either append to last element or add new element)

        // Loop through each chunk and process it
        let delimiter: u8 = 0x0A; // newline // TODO: read from config, or something
        for (chunk_num, chunk) in new_data.split(|&x| x == delimiter).enumerate() {
            let is_chunk_final = chunk.last() == Some(&delimiter);
            
            // This chunk_num check is an optimization to prevent looking into the last
            // element of the transfer log if it's not necessary.
            if chunk_num == 0 {
                if let Some(last_elem) = self.main_screen_transfer_log.last_mut() {
                    match last_elem.log_type {
                        TransferLogType::SerialData => {
                            match last_elem.is_final {
                                true => {
                                    self.main_screen_transfer_log.push(
                                        TransferLogEntry::new(
                                            chunk.to_vec(),
                                            TransferLogType::SerialData,
                                            is_chunk_final
                                        )
                                    );
                                }
                                false => {
                                    last_elem.data_as_bytes.extend(chunk);
                                    last_elem.is_final = is_chunk_final;
                                }
                            }
                        }
                        _ => {

                            self.main_screen_transfer_log.push(
                                TransferLogEntry::new(
                                    chunk.to_vec(),
                                    TransferLogType::SerialData,
                                    is_chunk_final
                                )
                            );
                        }
                    }
                }
            } else {
                self.main_screen_transfer_log.push(
                    TransferLogEntry::new(
                        chunk.to_vec(),
                        TransferLogType::SerialData,
                        is_chunk_final
                    )
                );
            }
        }
    }

    pub fn add_echo_to_transfer_log(&mut self, new_data: Vec<u8>) {
        if let Some(last_elem) = self.main_screen_transfer_log.last_mut() {
            last_elem.is_final = true;
        }
        self.main_screen_transfer_log.push(
            TransferLogEntry::new(
                new_data,
                TransferLogType::EchoData,
                true
            )
        );
    }

    pub fn add_error_to_transfer_log(&mut self, new_data: String) {
        if let Some(last_elem) = self.main_screen_transfer_log.last_mut() {
            last_elem.is_final = true;
        }
        self.main_screen_transfer_log.push(
            TransferLogEntry::new(
                new_data.into_bytes(),
                TransferLogType::ErrorData,
                true
            )
        );
    }
}

pub struct TransferLogEntry {
    pub data_as_bytes: Vec<u8>,
    pub log_type: TransferLogType,
    pub timestamp: chrono::DateTime<chrono::Local>,
    
    /// Whether this is the final entry in the log, or if it can still be written to.
    pub is_final: bool,
}

impl TransferLogEntry {
    pub fn new(data: Vec<u8>, log_type: TransferLogType, is_final: bool) -> Self {
        Self {
            data_as_bytes: data,
            log_type,
            timestamp: chrono::Local::now(),
            is_final,
        }
    }

    pub fn get_data_as_string(&self) -> String {
        let string_val = match self.log_type {
            TransferLogType::SerialData => {
                String::from_utf8(self.data_as_bytes.clone()).expect("Data should be UTF-8, for now")
            }
            TransferLogType::EchoData => {
                String::from_utf8(self.data_as_bytes.clone()).expect("Data should be UTF-8, for now")
            }
            TransferLogType::ErrorData => {
                String::from_utf8(self.data_as_bytes.clone()).expect("Error messages should be UTF-8")
            }
        }.trim_end().to_string();
        
        string_val
    }
}

pub enum TransferLogType {
    SerialData,
    EchoData,
    ErrorData,
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

