mod app;
use app::{App, CurrentScreen, AppConfig};

mod tui;
use ratatui::style::Modifier;
use serialport5::SerialPort;
use tui::ui;

mod tui_list_state_tracker;
use tui_list_state_tracker::ListStateTracker;

mod serial;
use serial::{bind_serial_port};

use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use crossterm::event::{DisableMouseCapture, KeyboardEnhancementFlags};
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;

use std::io;
use std::io::{Read, Write};

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    
    // enable capturing Ctrl+CHAR
    crossterm::event::PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    );
    crossterm::event::PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
    );

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(_) => eprintln!("run_app exited successfully"),
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    println!("Goodbye!");

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    

    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
            if let Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    
                    // Handle KeyEventKind::Press events:
                    if app_handle_keypresses(app, key) {
                        break;
                    }
                }
            }
        }

        // handle incoming serial data
        match &mut app.bound_serial_port {
            Some(port) => {
                match port.bytes_to_read() {
                    Ok(bytes_to_read) => {
                        if bytes_to_read > 0 {
                            app_handle_incoming_serial_data(app);
                        }
                    }
                    Err(e) => {
                        app.main_incoming_serial_data.push_str(&format!("Error checking bytes to read: {}", e));
                    }
                }
            }
            None => { }
        }
    }
    Ok(())
}

fn app_handle_incoming_serial_data(app: &mut App) -> () {
    // TODO: handle expect better maybe
    let port = app.bound_serial_port.as_mut().expect("Serial port unbound itself between seeing if bytes are available, and reading them.");

    let mut serial_buf: Vec<u8> = vec![0; 32];
    let bytes_read_count = port.read(serial_buf.as_mut_slice());

    match bytes_read_count {
        Ok(bytes_read_count) => {
            if bytes_read_count > 0 {
                let data = &serial_buf[..bytes_read_count];
                let data_str = std::str::from_utf8(data).expect("Invalid UTF-8 chars, FIXME"); // FIXME: horrible unwrap here; this will have issues
                app.main_incoming_serial_data.push_str(data_str); // TODO: delete very old data from this buffer to prevent memory leak
                // TODO: push the data with color formatting maybe (for different types of data [e.g., EOL, end-of-message, non-printable-as-hex, etc.])
                // TODO: write to files/logs, etc.
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            // do nothing
        }
        Err(e) => {
            app.main_incoming_serial_data.push_str(&format!("Error reading from serial port: {}", e));
        }
    }
}

/// Handle keypresses for the app (next-screen logic, quit logic, input logic, etc.)
/// Returns true if the app should exit
fn app_handle_keypresses(app: &mut App, key: KeyEvent) -> bool {
    match app.current_screen {
        CurrentScreen::PickSerialPort => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            // TODO: handle up and down arrows
            match key.code {
                KeyCode::Char('j') | KeyCode::Down | KeyCode::Right => {
                    app.pick_serial_port_list_state.next();
                }
                KeyCode::Char('k') | KeyCode::Up | KeyCode::Left => {
                    app.pick_serial_port_list_state.previous();
                }
                KeyCode::Enter => {
                    // read and store the selected serial port
                    let selected_port = app.pick_serial_port_list_state.get_selected();
                    match selected_port {
                        Some(port) => {
                            app.selected_serial_port = Some(port);
                            match app.app_config.baud_rate {
                                None | Some(0) => {
                                    app.current_screen = CurrentScreen::PickBaudRate;
                                }
                                _ => {
                                    // if the baud rate is already set, just go to the main screen
                                    app_transition_to_main(app);
                                }
                            }
                        }
                        None => {
                            app.selected_serial_port = None;
                            // don't change screens
                        }
                    }
                }
                _ => {}
            }
        },

        CurrentScreen::PickBaudRate => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            match extract_modified_key(key) {
                ModifierWrapper::Control(KeyCode::Backspace) | ModifierWrapper::Control(KeyCode::Char('h')) => {
                    // Ctrl+Backspace should clear the field; it comes in as Ctrl+H sometimes
                    app.pick_baud_rate_input_field.clear();
                },
                _ => { }
            }
            match key.code {
                KeyCode::Char('b') => {
                    // go back (both 'b' and Ctrl+B)
                    app.current_screen = CurrentScreen::PickSerialPort;
                }
                KeyCode::Char('c') => {
                    // clear
                    app.pick_baud_rate_input_field.clear();
                }
                KeyCode::Char(c) => {
                    if c.is_ascii_digit() {
                        // silly check to avoid writing a number with a leading zero
                        if (app.pick_baud_rate_input_field.len() > 0)
                                || (app.pick_baud_rate_input_field.len() == 0 && c != '0'){
                            app.pick_baud_rate_input_field.push(c);
                        }
                    }
                }
                KeyCode::Backspace => {
                    app.pick_baud_rate_input_field.pop();
                }
                KeyCode::Enter => {
                    // store the baud rate
                    let baud_rate = app.pick_baud_rate_input_field.parse::<u32>();
                    match baud_rate {
                        Ok(rate) => {
                            app.app_config.baud_rate = Some(rate);
                            app_transition_to_main(app);
                        }
                        Err(_) => {
                            // this shouldn't really happen, just clear the field and let them try again though
                            app.pick_baud_rate_input_field.clear();
                        }
                    }
                }
                _ => {}
            }
        },

        CurrentScreen::Main => {
            if is_keypress_quit_event(key, false) { // NOTE: don't quit on 'q'
                return true; // exit program
            }

            match extract_modified_key(key) {
                ModifierWrapper::Control(KeyCode::Char('?')) => {
                    app.current_screen = CurrentScreen::Help;
                }
                ModifierWrapper::Control(KeyCode::Char('b')) => {
                    // TODO: if the PickBaudRate screen was skipped, then going back should skip right to the PickSerialPort screen
                    app.current_screen = CurrentScreen::PickBaudRate;
                }
                _ => {}
            }

            if app.main_screen_active_region_is_input {
                
                match extract_modified_key(key) {
                    ModifierWrapper::Control(KeyCode::Char('h')) | ModifierWrapper::Control(KeyCode::Backspace) => {
                        // Ctrl+Backspace should delete the last word
                        app.main_input = app.main_input.trim_end().to_string(); // first, remove all trailing spaces

                        let last_space_idx = app.main_input.rfind(' ').unwrap_or(0);
                        app.main_input.truncate(last_space_idx);
                        
                        // add a trailing space if there's still text
                        if app.main_input.len() > 0 {
                            app.main_input.push(' ');
                        }
                    }
                    
                    ModifierWrapper::Normal(KeyCode::Esc) => {
                        // change active region
                        app.main_screen_active_region_is_input = false;
                    }
                    ModifierWrapper::Normal(KeyCode::Char(c)) => {
                        app.main_input.push(c);
                    }
                    ModifierWrapper::Normal(KeyCode::Backspace) => {
                        app.main_input.pop();
                    }
                    ModifierWrapper::Normal(KeyCode::Enter) => {
                        match &mut app.bound_serial_port {
                            Some(port) => {
                                let data = app.main_input.as_bytes();
                                match port.write(data) {
                                    Ok(_) => {
                                        // clear the input field
                                        app.main_input.clear();
                                    }
                                    Err(e) => {
                                        // TODO: handle this disconnect situation better - probably go back to port selection screen
                                        app.main_input.push_str(&format!("Error writing to serial port: {}", e));
                                    }
                                }
                            }
                            None => {
                                // this should never really happen
                                app.main_input.push_str("Error: Serial port unbound itself between seeing if bytes are available, and reading them.");
                            }
                        }
                    }
                    _ => {}
                }
            }
            else { // active region is the incoming data region (with scroll bars)
                match key.code {
                    KeyCode::Esc => {
                        // change active region
                        app.main_screen_active_region_is_input = true;
                    }

                    // TODO: change scroll bindings
                    // TODO: check scroll repeat rate
                    // TODO: add mouse binding
                    // Scroll array bindings
                    KeyCode::Char('j') | KeyCode::Down => {
                        app.main_screen_vertical_scroll_val = 
                            app.main_screen_vertical_scroll_val.saturating_add(1);
                        app.main_screen_vertical_scroll_state =
                            app.main_screen_vertical_scroll_state.position(app.main_screen_vertical_scroll_val);
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.main_screen_vertical_scroll_val = 
                            app.main_screen_vertical_scroll_val.saturating_sub(1);
                        app.main_screen_vertical_scroll_state =
                            app.main_screen_vertical_scroll_state.position(app.main_screen_vertical_scroll_val);
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        app.main_screen_horizontal_scroll_val = 
                            app.main_screen_horizontal_scroll_val.saturating_sub(1);
                        app.main_screen_horizontal_scroll_state =
                            app.main_screen_horizontal_scroll_state.position(app.main_screen_horizontal_scroll_val);
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        app.main_screen_horizontal_scroll_val = 
                            app.main_screen_horizontal_scroll_val.saturating_add(1);
                        app.main_screen_horizontal_scroll_state =
                            app.main_screen_horizontal_scroll_state.position(app.main_screen_horizontal_scroll_val);
                    }
                    
                    // TODO: home/end/pageup/pagedown


                    KeyCode::Enter => {
                        // TODO: send the data
                    }
                    _ => {}
                }
            }
        },

        CurrentScreen::Config1 => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    app.current_screen = CurrentScreen::Main;
                }
                // TODO: handle changing the config
                _ => {}
            }
        },

        CurrentScreen::Help => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            match key.code {
                KeyCode::Esc => {
                    app.current_screen = CurrentScreen::Main;
                }
                _ => {
                    app.current_screen = CurrentScreen::Main;
                }
            }
        },

    };
    false
}

/// Attempts to transition the app to the main screen by opening the serial port.
/// If an error occurs, the app will revert back to the serial port selection screen, with an error message.
fn app_transition_to_main(app: &mut App) -> () {
    // attempt to open the serial port
    match (&app.selected_serial_port, app.app_config.baud_rate) {
        (Some(port_name), Some(baud_rate)) => {
            match bind_serial_port(&port_name, baud_rate) {
                Ok(serial_port) => {
                    app.bound_serial_port = Some(serial_port);
                    app.current_screen = CurrentScreen::Main;
                }
                Err(e) => {
                    app.general_error_message = Some(format!("Error binding serial port: {}", e));
                    app.current_screen = CurrentScreen::PickSerialPort;
                }
            }
        }
        (None, _) => {
            // this should never really happen
            app.general_error_message = Some(format!("Error: No serial port selected"));
            app.current_screen = CurrentScreen::PickSerialPort;
        }
        (_, None) => {
            // this should never really happen
            app.general_error_message = Some(format!("Error: No baud rate selected")); // this message may not be shown as-is
            app.current_screen = CurrentScreen::PickBaudRate;
        }
    }
}

fn is_keypress_quit_event(key: KeyEvent, is_q_quit: bool) -> bool {
    if key.kind == event::KeyEventKind::Release {
        return false;
    }
    match extract_modified_key(key) {
        ModifierWrapper::Control(KeyCode::Char('c')) => {
            return true;
        }
        ModifierWrapper::Control(KeyCode::Char('q')) => {
            return true;
        }
        ModifierWrapper::Normal(KeyCode::Char(']')) => {
            return true;
        }
        ModifierWrapper::Normal(KeyCode::Char('q')) => {
            return is_q_quit;
        }
        _ => {}
    }
    false
}

fn extract_modified_key(key: KeyEvent) -> ModifierWrapper {
    match key.modifiers {
        KeyModifiers::CONTROL => {
            return ModifierWrapper::Control(key.code);
        }
        _ => {
            return ModifierWrapper::Normal(key.code);
        }
    }
}

// TODO: get this working
enum ModifierWrapper {
    Control(KeyCode),
    Normal(KeyCode),
}
