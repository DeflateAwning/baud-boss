mod app;
use app::{App, CurrentScreen, AppConfig};

mod tui;
use ratatui::style::Modifier;
use tui::ui;

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
                    match key.code {
                        KeyCode::Enter => {
                            app.current_screen = CurrentScreen::PickBaudRate;
                        }
                        _ => {}
                    }
                },

                CurrentScreen::PickBaudRate => {
                    if is_keypress_quit_event(key, true) {
                        break;
                    }
                    match key.code {
                        KeyCode::Enter => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        _ => {}
                    }
                },

                CurrentScreen::Main => {
                    if is_keypress_quit_event(key, false) { // NOTE: don't quit on 'q'
                        break;
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
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        KeyCode::Enter => {
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
    if is_q_quit && (key.code == KeyCode::Char('q')) {
        return true;
    }
    if key.modifiers == KeyModifiers::CONTROL {
        if vec![KeyCode::Char('c'), KeyCode::Char(']')].contains(&key.code) {
            return true;
        }
    }
    false
}
