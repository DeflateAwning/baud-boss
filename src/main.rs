mod app;
mod tui;
mod tui_list_state_tracker;
mod serial;

use app::{App, AppConfig, CurrentScreen, EchoMode, MainScreenActiveRegion, ScrollPosition};
use tui::ui;
use serial::bind_serial_port;

use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers};
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
    
    // TODO: make this configurable
    // DEBUG
    // Fill the display with a bunch of fake data
    for i in 0_i32..=120_i32 {
        let fake_data = format!("Fake Incoming Data, line {}/120: {}\n",
            i, ".".repeat((100_i32-i).abs() as usize));
        app.add_rxd_serial_data_to_transfer_log(fake_data.into_bytes());
    }

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

                    // TODO: setup these DEBUG methods as configuration options
                    // DEBUG
                    // app.main_incoming_serial_data.push_str(
                    //     &format!("\nKey Log: key.modifiers={:?}, key.code={:?}\n",
                    //     key.modifiers, key.code));

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
                        app.add_error_to_transfer_log(format!("Error checking bytes to read: {}", e));
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
                match std::str::from_utf8(data) {
                    Ok(data_str) => {
                        app.add_rxd_serial_data_to_transfer_log(data_str.to_string().into_bytes());
                    }
                    Err(_) => {
                        // FIXME: still show the data somehow (hex or similar)
                        app.add_error_to_transfer_log(
                            format!("Error converting incoming data to UTF-8"));
                    }
                }
                // TODO: delete very old data from this buffer to prevent memory leak
                // TODO: push the data with color formatting maybe (for different types of data [e.g., EOL, end-of-message, non-printable-as-hex, etc.])
                // TODO: write to files/logs, etc.
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            // do nothing
        }
        Err(e) => {
            app.add_error_to_transfer_log(
                format!("Error reading from serial port: {}", e));
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
            app_handle_keypresses_for_pick_serial_port_screen(app, key)
        },

        CurrentScreen::PickBaudRate => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            app_handle_keypresses_for_pick_baud_rate_screen(app, key);
        },

        CurrentScreen::Main => {
            if is_keypress_quit_event(key, false) { // NOTE: don't quit on 'q'
                return true; // exit program
            }
            app_handle_keypresses_for_main_screen(app, key);
        },

        CurrentScreen::Config1 => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            app_handle_keypresses_for_config1_screen(app, key);
        },

        CurrentScreen::Help => {
            if is_keypress_quit_event(key, true) {
                return true; // exit program
            }
            app_handle_keypresses_for_help_screen(app, key);
        },

    };
    false
}

fn app_handle_keypresses_for_pick_serial_port_screen(app: &mut App, key: KeyEvent) -> () {
    match key.code {
        // TODO: refactor this list of keys to a standard place (key_codes_left_and_up, key_codes_right_and_down)
        KeyCode::Char('h') | KeyCode::Char('k') | KeyCode::Up | KeyCode::Left => {
            app.pick_serial_port_list_state.previous();
        }
        KeyCode::Char('j') | KeyCode::Char('l') | KeyCode::Down | KeyCode::Right => {
            app.pick_serial_port_list_state.next();
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
}

fn app_handle_keypresses_for_pick_baud_rate_screen(app: &mut App, key: KeyEvent) -> () {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Backspace | KeyCode::Char('h')) => {
            // Ctrl+Backspace should clear the field; it comes in as Ctrl+H sometimes
            app.pick_baud_rate_input_field.clear();
        },
        (KeyModifiers::NONE, KeyCode::Char('b')) => {
            // go back (both 'b' and Ctrl+B)
            app.current_screen = CurrentScreen::PickSerialPort;
        }
        (KeyModifiers::NONE, KeyCode::Char('c')) => {
            // clear
            app.pick_baud_rate_input_field.clear();
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            if c.is_ascii_digit() {
                // silly check to avoid writing a number with a leading zero
                if (app.pick_baud_rate_input_field.len() > 0)
                        || (app.pick_baud_rate_input_field.len() == 0 && c != '0'){
                    app.pick_baud_rate_input_field.push(c);
                }
            }
        }
        // TODO: add arrow keys to move cursor
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            app.pick_baud_rate_input_field.pop();
        }
        (KeyModifiers::NONE, KeyCode::Enter) => {
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
}

fn app_handle_keypresses_for_main_screen(app: &mut App, key: KeyEvent) -> () {
    // key handler (main_screen_active_region-independent)
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('?')) => {
            app.current_screen = CurrentScreen::Help;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            // TODO: if the PickBaudRate screen was skipped, then going back should skip right to the PickSerialPort screen
            app.current_screen = CurrentScreen::PickBaudRate;
        }

        (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Tab) => {
            app.main_screen_active_region = app.main_screen_active_region.next();
        }
        (KeyModifiers::SHIFT, KeyCode::Tab) | (_, KeyCode::BackTab) => {
            app.main_screen_active_region = app.main_screen_active_region.prev();
        }
        _ => {}
    }

    match app.main_screen_active_region {
        MainScreenActiveRegion::Input => {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('h') | KeyCode::Backspace) => {
                    // Ctrl+Backspace should delete the last word
                    app.main_input = app.main_input.trim_end().to_string(); // first, remove all trailing spaces

                    let last_space_idx = app.main_input.rfind(' ').unwrap_or(0);
                    app.main_input.truncate(last_space_idx);
                    
                    // add a trailing space if there's still text
                    if app.main_input.len() > 0 {
                        app.main_input.push(' ');
                    }
                }

                // left and right arrow keys to move the cursor
                (KeyModifiers::NONE, KeyCode::Left) => {
                    // move the cursor left
                    match app.main_input_cursor_position {
                        Some(pos) => {
                            if pos > 0 {
                                app.main_input_cursor_position = Some(pos - 1);
                            }
                        }
                        None => {
                            if app.main_input.len() > 0 {
                                app.main_input_cursor_position = Some(app.main_input.len() - 1);
                            }
                        }
                    }
                }
                (KeyModifiers::NONE, KeyCode::Right) => {
                    // move the cursor right
                    match app.main_input_cursor_position {
                        Some(pos) => {
                            if pos < (app.main_input.len() - 1) {
                                app.main_input_cursor_position = Some(pos + 1);
                            }
                            else {
                                app.main_input_cursor_position = None;
                            }
                        }
                        None => {}
                    }
                }
                
                // home and end keys to jump the cursor
                (KeyModifiers::NONE, KeyCode::Home) => {
                    if app.main_input.len() > 0 {
                        app.main_input_cursor_position = Some(0);
                    }
                }
                (KeyModifiers::NONE, KeyCode::End) => {
                    app.main_input_cursor_position = None;
                }

                (KeyModifiers::NONE, KeyCode::Up) => {
                    // go back in the send history
                    app.main_input_cursor_position = None;
                    match app.main_input_send_history_index {
                        Some(index) => {
                            if index > 0 {
                                app.main_input_send_history_index = Some(index - 1);
                                app.main_input = app.main_input_send_history[index - 1].clone();
                            }
                        }
                        None => {
                            if app.main_input_send_history.len() > 0 {
                                app.main_input_send_history_index = Some(app.main_input_send_history.len() - 1);
                                app.main_input_typing_in_progress_but_not_sent = Some(app.main_input.clone());
                                app.main_input = app.main_input_send_history.last().unwrap_or(&String::new()).clone();
                            }
                        }
                    }
                }
                (KeyModifiers::NONE, KeyCode::Down) => {
                    app.main_input_cursor_position = None;
                    // go forward in the send history
                    match app.main_input_send_history_index {
                        Some(index) => {
                            if index < (app.main_input_send_history.len() - 1) {
                                app.main_input_send_history_index = Some(index + 1);
                                app.main_input = app.main_input_send_history[index + 1].clone();
                            }
                            else if index == (app.main_input_send_history.len() - 1) {
                                app.main_input_send_history_index = None;
                                app.main_input = app.main_input_typing_in_progress_but_not_sent.take().unwrap_or(String::new());
                            }
                            else {
                                panic!("Index out of bounds in send history");
                            }
                        }
                        None => {
                            // do nothing
                        }
                    }
                }
                

                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    match app.main_input_cursor_position {
                        Some(pos) => {
                            app.main_input.insert(pos, c);
                            app.main_input_cursor_position = Some(pos + 1);
                        }
                        None => {
                            app.main_input.push(c);
                        }
                    }
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    match app.main_input_cursor_position {
                        Some(main_input_cursor_position) => {
                            if (app.main_input.len() > 0) && (main_input_cursor_position > 0) {
                                app.main_input.remove(main_input_cursor_position - 1);
                                app.main_input_cursor_position = Some(main_input_cursor_position - 1);
                            }
                            if main_input_cursor_position == app.main_input.len() {
                                // if we're now back at the new end, mark as such
                                app.main_input_cursor_position = None
                            }
                        }
                        None => {
                            app.main_input.pop();
                        }
                    }
                }
                (KeyModifiers::NONE, KeyCode::Delete) => {
                    match app.main_input_cursor_position {
                        Some(main_input_cursor_position) => {
                            if main_input_cursor_position < app.main_input.len() {
                                app.main_input.remove(main_input_cursor_position);

                                if main_input_cursor_position == app.main_input.len() {
                                    app.main_input_cursor_position = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // TODO: control+enter and shift+enter should allow adding newlines within a text block
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    match &mut app.bound_serial_port {
                        Some(port) => {
                            let mut data_string = app.main_input.clone();
                            data_string.push_str(&app.app_config.end_of_line);
                            let data = data_string.as_bytes();

                            match port.write(data) {
                                Ok(_) => {
                                    app.main_input_send_history.push(app.main_input.clone());
                                    app.main_input_send_history_index = None;

                                    match app.app_config.echo_mode {
                                        EchoMode::On => {
                                            // TODO: show newline chars well
                                            app.add_echo_to_transfer_log(
                                                app.main_input.clone().into_bytes());
                                        }
                                        EchoMode::Off => {}
                                    }

                                    app.main_input.clear();
                                    app.main_input_cursor_position = None;
                                }
                                Err(e) => {
                                    // TODO: handle this disconnect situation better - probably go back to port selection screen
                                    app.main_input.push_str(&format!("Error writing to serial port: {}", e));
                                }
                            }

                        }
                        None => {
                            // this should never really happen
                            // TODO: move this error message to the other side
                            app.main_input.push_str("Error: Serial port unbound itself between seeing if bytes are available, and reading them.");
                        }
                    }
                }
                _ => {}
            }
        }
        
        MainScreenActiveRegion::OutputScrollBars => {
            match key.code {
                KeyCode::Esc => {
                    app.main_screen_active_region = MainScreenActiveRegion::Input;
                    // TODO: see if it feels right to actually jump to bottom if it's not already at the bottom here
                }

                // TODO: change scroll bindings
                // TODO: check scroll repeat rate
                // TODO: add mouse binding
                // TODO: handle moving in diagonal directions
                
                // Scroll array bindings
                KeyCode::Char('j') | KeyCode::Down => {
                    app.main_screen_vert_scroll_val = 
                        app.main_screen_vert_scroll_val.saturating_add(1);

                    app.main_screen_vert_scroll_pos = ScrollPosition::FinitePosition;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.main_screen_vert_scroll_val = 
                        app.main_screen_vert_scroll_val.saturating_sub(1);

                    app.main_screen_vert_scroll_pos = ScrollPosition::FinitePosition;
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    app.main_screen_horiz_scroll_val = 
                        app.main_screen_horiz_scroll_val.saturating_sub(1);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    app.main_screen_horiz_scroll_val = 
                        app.main_screen_horiz_scroll_val.saturating_add(1);
                }
                
                KeyCode::Home => {
                    app.main_screen_horiz_scroll_val = 0;
                }
                KeyCode::End => {
                    app.main_screen_horiz_scroll_val = u32::MAX as usize;
                }

                // TODO: add these PageUp/PageDown handlers to the ActiveRegion::Input as well
                KeyCode::PageUp => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        app.main_screen_vert_scroll_val = 0;
                        app.main_screen_vert_scroll_pos = ScrollPosition::FinitePosition;
                    }
                    else {
                        app.main_screen_vert_scroll_val = 
                            app.main_screen_vert_scroll_val.saturating_sub(10);
                        
                        app.main_screen_vert_scroll_pos = ScrollPosition::FinitePosition;
                    }
                }
                KeyCode::PageDown => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        app.main_screen_vert_scroll_pos = ScrollPosition::PinnedAtEnd;
                        // This way isn't very good, but it worked, so leaving it here anyway
                        // app.main_screen_vertical_scroll_val = 0xFFF
                    }
                    else {
                        app.main_screen_vert_scroll_val = 
                            app.main_screen_vert_scroll_val.saturating_add(10);
                        app.main_screen_vert_scroll_pos = ScrollPosition::FinitePosition;
                    }
                }


                KeyCode::Enter => {
                    // TODO: send the data
                    // FIXME: confirm this can be removed; I think it can
                }
                _ => {}
            }
        }
    
        MainScreenActiveRegion::InputEolChoice => {
            // left and right cycle through the choices
            let eol_choices = vec!["", "\n", "\r", "\r\n", "\n\r"];
            let current_eol_idx: Option<usize> = eol_choices.iter().position(
                |&eol| eol == app.app_config.end_of_line);
            
            // Up, Left - move 
            match key.code {
                KeyCode::Esc => {
                    // change active region
                    app.main_screen_active_region = MainScreenActiveRegion::Input;
                }

                KeyCode::Left => {
                    app.app_config.end_of_line = eol_choices[
                        if current_eol_idx.unwrap_or(0) > 0 {
                            current_eol_idx.expect("Wrapping if statement lied") - 1
                        } else {
                            eol_choices.len() - 1
                        }
                    ].to_string();
                }
                KeyCode::Right => {
                    app.app_config.end_of_line = eol_choices[
                        current_eol_idx.unwrap().saturating_add(1) % eol_choices.len()
                    ].to_string();
                }
                _ => {}
            }
        }
    }
}

fn app_handle_keypresses_for_config1_screen(app: &mut App, key: KeyEvent) -> () {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => {
            app.current_screen = CurrentScreen::Main;
        }
        // TODO: handle changing the config
        _ => {}
    }
}

fn app_handle_keypresses_for_help_screen(app: &mut App, key: KeyEvent) -> () {
    match key.code {
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {
            app.current_screen = CurrentScreen::Main;
        }
    }
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
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            return true;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
            return true;
        }
        (KeyModifiers::NONE, KeyCode::Char(']')) => {
            return true;
        }
        (KeyModifiers::NONE, KeyCode::Char('q')) => {
            return is_q_quit;
        }
        _ => {}
    }
    false
}
