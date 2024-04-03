use crate::app::{App, CurrentScreen};

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph};
use ratatui::widgets::{Block, Borders, Wrap};
use ratatui::Frame;




pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = format!("Serial Port Terminal v{}", env!("CARGO_PKG_VERSION"));
    let title = Text::styled(title, Style::default().fg(Color::Yellow));
    f.render_widget(title, chunks[0]);

    // TODO: update global keybinding coloring
    let paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Help: Ctrl+H")
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[2]);

    match app.current_screen {
        CurrentScreen::PickSerialPort => {
            let ports = serialport5::available_ports();
            
            let text = match ports {
                Ok(ports) => {
                    Text::raw(format!("Available serial ports ({} ports):", ports.len()))
                    // TODO: print the selectable list of ports
                },
                Ok(_) => Text::raw("No serial ports found!"),
                Err(e) => Text::raw(format!("Error listing serial ports: {}", e)),
            };
            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Serial Ports"))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        },
        CurrentScreen::PickBaudRate => {
            let paragraph = Paragraph::new(Text::raw("Pick a baud rate (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Baud Rates"))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        },
        CurrentScreen::Main => {
            let paragraph = Paragraph::new(Text::raw("Main Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Main"))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        },
        CurrentScreen::Config1 => {
            let paragraph = Paragraph::new(Text::raw("Config1 Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Config 1"))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        },
        CurrentScreen::Help => {
            let paragraph = Paragraph::new(Text::raw("Help Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        },
    }
}
