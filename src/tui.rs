use std::cmp::{max, min};

use crate::app::{App, CurrentScreen, MainScreenActiveRegion, VisibleTransferData, ScrollPosition};
use crate::tui_list_state_tracker::ListStateTracker;

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::scrollbar;
use ratatui::text::{Line, Masked, Text};
use ratatui::widgets::block::{Position, Title};
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
            // TODO: wrap the EOL character in a box with borders, maybe (and/or highlight the borders when MainScreenActiveRegion::InputEolChoice)
            let eol_as_repr_string = app.app_config.end_of_line
                .replace("\n", "\\n").replace("\r", "\\r");
            let send_input_paragraph_lines: Vec<Line> = vec![
                Line::from(send_input_text),
            ];
            let send_input_paragraph = Paragraph::new(send_input_paragraph_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(match app.main_screen_active_region {
                            MainScreenActiveRegion::Input => {
                                Style::default().fg(Color::Green)
                            }
                            _ => Style::default()
                        }) // Set border color to red
                        .title("Send Data") // TODO: when it overflows, somewhere make sure we're always at the bottom of the content
                        .title(
                            Title::default()
                                .content(format!("EOL: '{}'", eol_as_repr_string))
                                .position(Position::Bottom)
                                .alignment(Alignment::Right)
                                // .style(match app.main_screen_active_region {
                                //     MainScreenActiveRegion::InputEolChoice =>
                                //         Style::default().fg(Color::Green),
                                //     _ => Style::default().fg(Color::Gray)
                                // })
                        )
                )
                .wrap(Wrap { trim: false });
            // TODO: if showing history, show the negative index of the history in the "Send Data" text
            // TODO: show if immediate or on-enter
            // TODO: show if hex or ascii
            // TODO: in immediate mode, flash the characters here for a sec after sending (maybe 500ms)


            let size = frame.size();
            let total_screen_height = size.height;
            let input_box_width = size.width - 2; // subtract 2 for the borders
            let required_lines_for_send_input_paragraph = send_input_paragraph.line_count(input_box_width);
            let required_send_input_height: u16 = (required_lines_for_send_input_paragraph as u16).max(1) + 2_u16;
            
            // top_element_height should be from 3 (min) to required_send_input_height+2 (ideal),
            //  but should be limited to `total_screen_height*0.4`
            let top_element_height: u16 = min(
                (total_screen_height as f32 * 0.6).round() as u16,
                required_send_input_height,
            ).max(3); // 3 is the absolute minimum height for the top element
            // TODO: deal with case where top_element_height < required_send_input_height here (scrolling indicator, etc.)

            // let debug_str: String = format!("DEBUG: required_lines_for_send_input_paragraph={}, required_send_input_height={}, top_element_height={} \n", 
            //     required_lines_for_send_input_paragraph,
            //     required_send_input_height, top_element_height);
            // app.main_incoming_serial_data.insert_str(0, debug_str.as_str());
            let main_screen_chunks = Layout::vertical([
                Constraint::Length(top_element_height),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(size);

            frame.render_widget(send_input_paragraph, main_screen_chunks[0]);

            let main_title_text = format!("Port '{}' @ {} baud", app.selected_serial_port.clone().unwrap_or_default(), app.app_config.baud_rate.unwrap_or_default());

            // FIXME: START HERE
            // need to convert these to Lines, and also get the metadata for the longest line
            let transfer_log_lines_as_strs: Vec<String> = app.main_screen_transfer_log
                .iter()
                .map(|line| match line {
                    VisibleTransferData::SerialData(data) => data.clone(),
                    VisibleTransferData::EchoData(data) => data.clone(),
                    VisibleTransferData::ErrorData(data) => data.clone(),
                }).collect();
            let longest_line_length = transfer_log_lines_as_strs.iter().map(|line| line.len()).max().unwrap_or(0);
            let transfer_log_lines: Vec<Line> = app.main_screen_transfer_log
                .iter()
                .map(|incoming_data_type| match incoming_data_type {
                    VisibleTransferData::SerialData(data) => {
                        Line::from(data.clone())
                            .style(Style::default()) //.fg(Color::Green))
                    },
                    VisibleTransferData::EchoData(data) => {
                        Line::from(data.clone())
                            .style(Style::default().fg(Color::LightBlue))
                    },
                    VisibleTransferData::ErrorData(data) => {
                        Line::from(data.clone())
                            .style(Style::default().fg(Color::Red))
                    },
                }).collect();

            app.main_screen_vert_scroll_state = app.main_screen_vert_scroll_state
                .content_length(transfer_log_lines.len());
            app.main_screen_horiz_scroll_state = app.main_screen_horiz_scroll_state
                .content_length(longest_line_length);

            // Scrollbar Rendering Examples: https://github.com/ratatui-org/ratatui/blob/main/examples/scrollbar.rs
            // TODO: prevent scrolling if there's no need to scroll (currently lets you scroll the content fully off the screen)
                // https://github.com/ratatui-org/ratatui/issues/1017
            
            // TODO: add config option for wrapping text in the incoming_data block
        
            let transfer_log_paragraph = Paragraph::new(transfer_log_lines)
                // .gray()
                .block(Block::default().borders(Borders::ALL).title(main_title_text.bold()))
                .scroll((
                    app.main_screen_vert_scroll_val as u16,
                    app.main_screen_horiz_scroll_val as u16));

            // TODO: refactor this clamping into a function, and try to apply it at the keyhandler level remove the jitter/vibrate
            // Right before rendering, perform a check on the vertical scroll position.
            // Clamp the value so the last line is at the bottom of the block.
            let incoming_data_paragraph_width = size.width - 2; // subtract 2 for the borders
            let incoming_data_paragraph_lines_count = transfer_log_paragraph.line_count(incoming_data_paragraph_width);
            let incoming_data_viewport_height: usize = main_screen_chunks[1].height as usize - 2; // subtract 2 for borders
            let max_vertical_scroll_val = (
                (incoming_data_paragraph_lines_count as i64)
                    - (incoming_data_viewport_height as i64)
                    + 1 // show an empty line at the bottom, why not
                ).max(0) as usize;
            if (app.main_screen_vert_scroll_val + incoming_data_viewport_height) > incoming_data_paragraph_lines_count {
                // do the clamp to the bottom/end of the scroll region
                app.main_screen_vert_scroll_pos = ScrollPosition::PinnedAtEnd;
                app.main_screen_vert_scroll_val = max_vertical_scroll_val;
                // app.main_screen_vert_scroll_state.last(); // TODO: maybe we should use this thing instead? Idk
            } // FIXME: it vibrates when it's at the bottom and you're typing in the input box
            if app.main_screen_vert_scroll_pos == ScrollPosition::PinnedAtEnd {
                app.main_screen_vert_scroll_val = max_vertical_scroll_val;
            }

            // Right before rendering, perform a check on the horizontal scroll position.
            // Clamp the value so the last character is at the right of the block.
            let incoming_data_paragraph_longest_line_length = transfer_log_lines_as_strs.iter().map(|line| line.len()).max().unwrap_or(0);
            let incoming_data_viewport_width: usize = main_screen_chunks[1].width as usize - 2; // subtract 2 for borders
            if (app.main_screen_horiz_scroll_val + incoming_data_viewport_width) > incoming_data_paragraph_longest_line_length {
                app.main_screen_horiz_scroll_val = (
                    (incoming_data_paragraph_longest_line_length as i64)
                        - (incoming_data_viewport_width as i64)
                        + 1 // show an empty col at the right, why not
                    ).max(0) as usize;
            } // FIXME: it vibrates when it's at the right and you're typing in the input box

            // create the scrollbar state
            app.main_screen_vert_scroll_state =
                app.main_screen_vert_scroll_state.position(app.main_screen_vert_scroll_val);
            app.main_screen_horiz_scroll_state =
                app.main_screen_horiz_scroll_state.position(app.main_screen_horiz_scroll_val);

            frame.render_widget(transfer_log_paragraph, main_screen_chunks[1]);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .thumb_style(match app.main_screen_active_region {
                        MainScreenActiveRegion::OutputScrollBars => {
                            Style::default().fg(Color::LightGreen)
                        }
                        _ => Style::default()
                    })
                    .begin_symbol(Some("↑")).end_symbol(Some("↓"))
                    .thumb_symbol("░")
                    .hide_when_not_scrollable(true)
                    .include_overscroll(false), // TOOD: check veritcal thumb symbol
                main_screen_chunks[1],
                &mut app.main_screen_vert_scroll_state,
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
                    .thumb_symbol("░")
                    .hide_when_not_scrollable(true)
                    .include_overscroll(false),
                main_screen_chunks[1].inner(&Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
                &mut app.main_screen_horiz_scroll_state,
            );

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
