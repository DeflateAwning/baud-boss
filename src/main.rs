mod app;
use app::{App, CurrentScreen, AppConfig};

mod tui;
use ratatui::style::Modifier;
use tui::ui;

mod tui_list_state_tracker;
use tui_list_state_tracker::ListStateTracker;

use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;

use crossterm::event::{DisableMouseCapture, KeyboardEnhancementFlags};
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

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

        
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::PickSerialPort => {
                    if is_keypress_quit_event(key, true) {
                        break;
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
                                    app.current_screen = CurrentScreen::PickBaudRate;
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
                        break;
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
                            // go back
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
                                    app.app_config.baud_rate = rate;
                                    app.current_screen = CurrentScreen::Main;
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
                        break;
                    }

                    match extract_modified_key(key) {
                        ModifierWrapper::Control(KeyCode::Char('h')) => {
                            app.current_screen = CurrentScreen::Help;
                        }
                        ModifierWrapper::Control(KeyCode::Char('b')) => {
                            app.current_screen = CurrentScreen::PickBaudRate;
                        }
                        _ => {}
                    }

                    match key.code {
                        KeyCode::Enter => {
                            // TODO: send the data
                        }
                        _ => {}
                    }
                },

                CurrentScreen::Config1 => {
                    if is_keypress_quit_event(key, true) {
                        break;
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
                        break;
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

            }

        }
    }
    Ok(())
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
