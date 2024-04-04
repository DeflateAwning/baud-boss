use crate::app::{App, CurrentScreen, MainScreenActiveRegion};
use crate::tui_list_state_tracker::ListStateTracker;

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::scrollbar;
use ratatui::text::{Line, Masked, Text};
use ratatui::widgets::{List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation};
use ratatui::widgets::{Block, Borders, Wrap};
use ratatui::Frame;
use ratatui::text::{Span};

// traits
use ratatui::style::Stylize;



pub fn ui(frame: &mut Frame, app: &mut App) {
    let general_chunks = Layout::default()
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
        .split(frame.size());

    let title = format!("Baud Boss Serial Terminal v{}", env!("CARGO_PKG_VERSION"));
    let title = Text::styled(title, Style::default().fg(Color::Yellow));

    match app.current_screen {
        CurrentScreen::PickSerialPort => {
            let ports = serialport5::available_ports();
            let port_count = match &ports {
                Ok(ports) => ports.len(),
                _ => 0,
            };

            let mut list_items_strs: Vec<String> = match &ports {
                Ok(ports) => {
                    if ports.is_empty() {
                        vec![String::from("No serial ports found!")] // FIXME: make this not selectable
                    }
                    else {
                        ports.iter().map(|port| {
                            format!("{}", port.port_name)
                        }).collect()
                    }
                },
                Err(e) => vec![format!("Error finding serial ports ({})!", e)] // FIXME: make this not selectable
            };

            if app.general_error_message.is_some() {
                list_items_strs.push(app.general_error_message.clone().unwrap()); // FIXME: make this not selectable (it's a hack for now)
            }

            app.pick_serial_port_list_state.update_items(list_items_strs.clone());
            
            // let display_items: Vec<ListItem> = app.pick_serial_port_list_state.get_as_list_items().clone(); // TODO: make this work (it'd be more elegant)
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
            
            frame.render_widget(title, general_chunks[0]);
            frame.render_stateful_widget(port_select_block, general_chunks[1], &mut app.pick_serial_port_list_state.state);


            // TODO: update keybinding coloring, update based on which screen is active
            let help_paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Help: Ctrl+H")
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });
            frame.render_widget(help_paragraph, general_chunks[2]);
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
            
            // TODO: make the baud rate selection show a greyed-out default value, which is selectable with 'enter'
            let baud_input_text = format!("Value: {}{} bits per second (baud)", app.pick_baud_rate_input_field, get_blinking_cursor(' ', '_'));
            let baud_input_paragraph = Paragraph::new(Text::raw(baud_input_text))
                .block(Block::default().borders(Borders::ALL).title(select_baud_rate_title_text))
                .wrap(Wrap { trim: true });

            // TODO: update keybinding coloring, update based on which screen is active
            let help_paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Help: Ctrl+H")
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });

            frame.render_widget(title, general_chunks[0]);
            frame.render_widget(baud_input_paragraph, general_chunks[1]);
            frame.render_widget(help_paragraph, general_chunks[2]);
        },
        CurrentScreen::Main => {
            // TODO: add option to show hex-and-ascii side-by-side
            let size = frame.size();
            let main_screen_chunks = Layout::vertical([
                Constraint::Length(3), // TODO: make the input part grow as needed, but be as small as possible
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

            let main_title_text = format!("Port '{}' @ {} baud", app.selected_serial_port.clone().unwrap_or_default(), app.app_config.baud_rate.unwrap_or_default());
            // let paragraph = Paragraph::new(Text::raw(&app.main_incoming_serial_data))
            //     .block(Block::default().borders(Borders::ALL).title(main_title_text))
            //     .wrap(Wrap { trim: true });

            let was_incoming_data_scroll_at_bottom: bool = app.main_screen_vertical_scroll_state.is_at_bottom();

            let incoming_data_lines_as_strs: Vec<String> = app.main_incoming_serial_data.lines().map(|line| line.to_string()).collect();
            let longest_line_length = incoming_data_lines_as_strs.iter().map(|line| line.len()).max().unwrap_or(0);
            let incoming_data_lines: Vec<Line> = incoming_data_lines_as_strs.iter().map(|line| Line::from(line.clone())).collect();

            app.main_screen_vertical_scroll_state = app.main_screen_vertical_scroll_state.content_length(incoming_data_lines.len());
            app.main_screen_horizontal_scroll_state = app.main_screen_horizontal_scroll_state.content_length(longest_line_length);
            
            let send_input_text = match app.main_input_cursor_position {
                Some(cursor_position) => {
                    let mut input_text = app.main_input.clone();
                    match app.main_screen_active_region {
                        MainScreenActiveRegion::Input => {
                            input_text.insert(cursor_position, get_blinking_cursor('|', ' '));
                        }
                        _ => {}
                    }
                    input_text
                },
                None => {
                    match app.main_screen_active_region {
                        MainScreenActiveRegion::Input => {
                            format!("{}{}", app.main_input, get_blinking_cursor('_', ' '))
                        }
                        _ => app.main_input.clone()
                    }
                }
            };
            let send_input_paragraph = Paragraph::new(Text::raw(send_input_text))
                .block(Block::default()
                .borders(Borders::ALL)
                .border_style(match app.main_screen_active_region {
                    MainScreenActiveRegion::Input => {
                        Style::default().fg(Color::Green)
                    }
                    _ => Style::default()
                }) // Set border color to red
                .title("Send Data"))
                .wrap(Wrap { trim: false });
            // TODO: if showing history, show the negative index of the history in the "Send Data" text
            // TODO: show if immediate or on-enter
            // TODO: show if hex or ascii
            // TODO: in immediate mode, flash the characters here for a sec after sending (maybe 500ms)
            frame.render_widget(send_input_paragraph, main_screen_chunks[0]);

            // Scrollbar Rendering Examples: https://github.com/ratatui-org/ratatui/blob/main/examples/scrollbar.rs
            // TODO: prevent scrolling if there's no need to scroll (currently lets you scroll the content fully off the screen)
                // https://github.com/ratatui-org/ratatui/issues/1017
            
            // TODO: add config option for wrapping text in the incoming_data block
        
            let incoming_data_paragraph = Paragraph::new(incoming_data_lines.clone())
                // .gray()
                .block(Block::default().borders(Borders::ALL).title(main_title_text.bold()))
                .scroll((
                    app.main_screen_vertical_scroll_val as u16,
                    app.main_screen_horizontal_scroll_val as u16));

            frame.render_widget(incoming_data_paragraph, main_screen_chunks[1]);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .thumb_style(match app.main_screen_active_region {
                        MainScreenActiveRegion::OutputScrollBars => {
                            Style::default().fg(Color::LightGreen)
                        }
                        _ => Style::default()
                    })
                    .begin_symbol(Some("↑")).end_symbol(Some("↓"))
                    .thumb_symbol("░"), // TOOD: check veritcal thumb symbol
                main_screen_chunks[1],
                &mut app.main_screen_vertical_scroll_state,
            );
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                    .thumb_style(match app.main_screen_active_region {
                        MainScreenActiveRegion::OutputScrollBars => {
                            Style::default().fg(Color::LightGreen)
                        }
                        _ => Style::default()
                    })
                    .begin_symbol(Some("◄")).end_symbol(Some("►"))
                    .thumb_symbol("░"),
                main_screen_chunks[1].inner(&Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
                &mut app.main_screen_horizontal_scroll_state,
            );

            // scroll to the new bottom, if we're already near the bottom
            if was_incoming_data_scroll_at_bottom {
                app.main_screen_vertical_scroll_state.last();
            }

            // TODO: update keybinding coloring, update based on which screen is active
            let help_paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Type to prep data | Enter to send")
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });
            frame.render_widget(help_paragraph, main_screen_chunks[2]);
        },
        CurrentScreen::Config1 => {
            let paragraph = Paragraph::new(Text::raw("Config1 Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Config 1"))
                .wrap(Wrap { trim: true });

            frame.render_widget(title, general_chunks[0]);
            frame.render_widget(paragraph, general_chunks[1]);


            // TODO: update keybinding coloring, update based on which screen is active
            let help_paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Help: Ctrl+H")
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });
            frame.render_widget(help_paragraph, general_chunks[2]);
        },
        CurrentScreen::Help => {
            let paragraph = Paragraph::new(Text::raw("Help Screen (NOT IMPLEMENTED):"))
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });

            frame.render_widget(title, general_chunks[0]);
            frame.render_widget(paragraph, general_chunks[1]);

            // TODO: update keybinding coloring, update based on which screen is active
            let help_paragraph = Paragraph::new("Quit: Ctrl+] or Ctrl+C | Menu: Ctrl+T | Help: Ctrl+H")
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .wrap(Wrap { trim: true });
            frame.render_widget(help_paragraph, general_chunks[2]);
        },
    }
}

fn get_blinking_cursor(on_symbol: char, off_symbol: char) -> char {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let since_the_epoch = since_the_epoch.as_millis();
    if since_the_epoch % 1000 < 500 {
        on_symbol
    }
    else {
        off_symbol
    }
}
