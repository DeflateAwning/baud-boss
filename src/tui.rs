use crate::app::{App, CurrentScreen};
use crate::tui_list_state_tracker::ListStateTracker;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{List, ListItem, Paragraph};
use ratatui::widgets::{Block, Borders, Wrap};
use ratatui::Frame;
use ratatui::text::{Span};




pub fn ui(f: &mut Frame, app: &mut App) {
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

    let title = format!("Baud Boss Serial Terminal v{}", env!("CARGO_PKG_VERSION"));
    let title = Text::styled(title, Style::default().fg(Color::Yellow));
    

    // TODO: update global keybinding coloring, update based on which screen is active
    let paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Help: Ctrl+H")
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[2]);

    match app.current_screen {
        CurrentScreen::PickSerialPort => {
            let ports = serialport5::available_ports();
            let port_count = match &ports {
                Ok(ports) => ports.len(),
                _ => 0,
            };

            let list_items_strs: Vec<String> = match &ports {
                Ok(ports) => {
                    if ports.is_empty() {
                        vec![String::from("No serial ports found!")] // TODO: make this not selectable
                    }
                    else {
                        ports.iter().map(|port| {
                            format!("{}", port.port_name)
                        }).collect()
                    }
                },
                Err(e) => vec![format!("Error finding serial ports ({})!", e)] // TODO: make this not selectable
            };

            app.pick_serial_port_list_state.update_items(list_items_strs.clone());
            
            // let display_items: Vec<ListItem> = app.pick_serial_port_list_state.get_as_list_items().clone(); // TODO: make this work
            let display_items: Vec<ListItem> = app.pick_serial_port_list_state.items.iter().map(|port| {
                ListItem::new(Span::raw(format!("{}", port)))
            }).collect();

            // TODO: make the rescan message prettier
            let select_port_text = format!("Select a serial port ({} ports found) [automatically rescans constantly]:", port_count);

            // Create a List from the port items
            let port_select_block = List::new(display_items)
                .block(Block::default().borders(Borders::ALL).title(select_port_text))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> "); // Optional: indicates the selected item
            
            f.render_widget(title, chunks[0]);
            f.render_stateful_widget(port_select_block, chunks[1], &mut app.pick_serial_port_list_state.state);
        },
        CurrentScreen::PickBaudRate => {
            let select_baud_rate_title_text: String = match app.selected_serial_port.clone() {
                Some(selected_serial_port) => {
                    format!("Pick a baud rate (for {}):", selected_serial_port)
                },
                None => {
                    format!("ERROR: No port selected. Please go back and select a port first.")
                    // this should never really happen
                }
            };
            
            let paragraph = Paragraph::new(Text::raw("TODO: type the port here"))
                .block(Block::default().borders(Borders::ALL).title(select_baud_rate_title_text))
                .wrap(Wrap { trim: true });

            f.render_widget(title, chunks[0]);
            f.render_widget(paragraph, chunks[1]);
        },
        CurrentScreen::Main => {
            let paragraph = Paragraph::new(Text::raw("Main Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Main"))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);

            // TODO: render input box
        },
        CurrentScreen::Config1 => {
            let paragraph = Paragraph::new(Text::raw("Config1 Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Config 1"))
                .wrap(Wrap { trim: true });

            f.render_widget(title, chunks[0]);
            f.render_widget(paragraph, chunks[1]);
        },
        CurrentScreen::Help => {
            let paragraph = Paragraph::new(Text::raw("Help Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });

            f.render_widget(title, chunks[0]);
            f.render_widget(paragraph, chunks[1]);
        },
    }
}
