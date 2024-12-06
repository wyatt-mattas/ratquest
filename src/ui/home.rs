use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use std::rc::Rc;

use crate::app::{ActivePanel, App, CurrentScreen, DetailField, Groups};
use crate::ui::popups::{
    add_request_popup, editing_popup, exiting_popup, render_header_popup, render_params_popup,
};

pub fn ui(frame: &mut Frame, app: &mut App) {
    // First render all the regular UI elements
    render_base_ui(frame, app);

    // Then render any popups on top
    if app.adding_header {
        render_header_popup(frame, app);
    }

    if app.adding_params {
        render_params_popup(frame, app);
    }
}

fn title_block_component(frame: &mut Frame, chunks: &Rc<[Rect]>) {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "API Groups",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, chunks[0]);
}

fn render_tree(frame: &mut Frame, app: &mut App, chunks: &Rc<[Rect]>) {
    let tree_block = Block::default()
        .borders(Borders::ALL)
        .title("API Groups")
        .border_style(if app.active_panel == ActivePanel::Tree {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let tree_area = tree_block.inner(chunks[0]);
    frame.render_widget(tree_block, chunks[0]);
    app.render_tree_view(frame, tree_area);
}

fn render_base_ui(frame: &mut Frame, app: &mut App) {
    let chunks: std::rc::Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Title
    title_block_component(frame, &chunks);

    // Main body layout
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[1]);

    render_tree(frame, app, &inner_layout);

    // Update the details block:
    let details_block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .border_style(if app.active_panel == ActivePanel::Details {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner_area = details_block.inner(inner_layout[1]);
    frame.render_widget(details_block, inner_layout[1]);

    // Render the details view in the right panel
    if let Some(request) = app.get_current_request() {
        // Create the layout for the right panel within the block
        let details_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // URL
                Constraint::Length(6),  // Body
                Constraint::Length(8),  // Parameters
                Constraint::Length(8),  // Headers
                Constraint::Length(10), // Auth
                Constraint::Length(3),  // Send Request Bar
                Constraint::Min(0),     // Response Area
            ])
            .split(inner_area);

        // URL Section
        let url_block = Block::default()
            .borders(Borders::ALL)
            .title("URL")
            .border_style(if app.current_detail_field == DetailField::Url {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let url_area = url_block.inner(details_layout[0]);
        frame.render_widget(url_block, details_layout[0]);

        // Only show cursor if this field is selected
        if app.current_detail_field == DetailField::Url {
            frame.render_widget(&app.url_textarea, url_area);
        } else {
            // When not selected, render as a regular paragraph without cursor
            frame.render_widget(
                Paragraph::new(app.url_textarea.lines().join("\n")).style(Style::default()),
                url_area,
            );
        }

        // Body Section
        let body_block = Block::default()
            .borders(Borders::ALL)
            .title("Body")
            .border_style(if app.current_detail_field == DetailField::Body {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let body_area = body_block.inner(details_layout[1]);
        frame.render_widget(body_block, details_layout[1]);

        if app.current_detail_field == DetailField::Body {
            frame.render_widget(&app.body_textarea, body_area);
        } else {
            frame.render_widget(
                Paragraph::new(app.body_textarea.lines().join("\n")).style(Style::default()),
                body_area,
            );
        }

        // Regular headers display (your existing code)
        let params_text = request
            .details
            .params
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let params = Paragraph::new(params_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Parameters (Enter to add)")
                .border_style(if app.current_detail_field == DetailField::Params {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        );

        frame.render_widget(params, details_layout[2]);

        // Regular headers display (your existing code)
        let headers_text = request
            .details
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let headers = Paragraph::new(headers_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Headers (Enter to add)")
                .border_style(if app.current_detail_field == DetailField::Headers {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        );

        frame.render_widget(headers, details_layout[3]);

        // Auth Section
        let auth_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Auth Type
                Constraint::Min(1),    // Auth Details
            ])
            .split(details_layout[4]);

        // Auth Type
        let auth_type_text = format!("Auth Type: {}", request.details.auth_type.as_str());
        let auth_type = Paragraph::new(auth_type_text).block(
            Block::default().borders(Borders::ALL).border_style(
                if app.current_detail_field == DetailField::AuthType {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            ),
        );
        frame.render_widget(auth_type, auth_layout[0]);

        // Auth Details
        match request.details.auth_type {
            crate::app::AuthType::None => {
                let no_auth = Paragraph::new("No authentication required")
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(no_auth, auth_layout[1]);
            }
            crate::app::AuthType::Basic => {
                let basic_auth_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Username
                        Constraint::Length(3), // Password
                    ])
                    .split(auth_layout[1]);

                // Username
                let username_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Username")
                    .border_style(if app.current_detail_field == DetailField::AuthUsername {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });

                let username_area = username_block.inner(basic_auth_layout[0]);
                frame.render_widget(username_block, basic_auth_layout[0]);

                if app.current_detail_field == DetailField::AuthUsername {
                    frame.render_widget(&app.auth_username_textarea, username_area);
                } else {
                    frame.render_widget(
                        Paragraph::new(app.auth_username_textarea.lines().join("\n"))
                            .style(Style::default())
                            .wrap(Wrap { trim: true }),
                        username_area,
                    );
                }

                // Password
                let password_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Password")
                    .border_style(if app.current_detail_field == DetailField::AuthPassword {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });

                let password_area = password_block.inner(basic_auth_layout[1]);
                frame.render_widget(password_block, basic_auth_layout[1]);

                // if app.current_detail_field == DetailField::AuthPassword {
                //     if app.password_visible {
                //         frame.render_widget(&app.auth_password_textarea, password_area);
                //     } else {
                //         // Show dots with cursor when selected but not visible
                //         let masked_text = app
                //             .auth_password_textarea
                //             .lines()
                //             .iter()
                //             .map(|line| "•".repeat(line.len()))
                //             .collect::<Vec<_>>()
                //             .join("\n");
                //         frame.render_widget(
                //             Paragraph::new(masked_text)
                //                 .style(Style::default())
                //                 .wrap(Wrap { trim: true }),
                //             password_area,
                //         );
                //     }
                // } else {
                //     // Not selected - always show as paragraph with dots unless visibility is enabled
                //     let password_text = if app.password_visible {
                //         app.auth_password_textarea.lines().join("\n")
                //     } else {
                //         app.auth_password_textarea
                //             .lines()
                //             .iter()
                //             .map(|line| "•".repeat(line.len()))
                //             .collect::<Vec<_>>()
                //             .join("\n")
                //     };

                //     frame.render_widget(
                //         Paragraph::new(password_text)
                //             .style(Style::default())
                //             .wrap(Wrap { trim: true }),
                //         password_area,
                //     );
                // }

                // Create masked password text
                let password_text = if app.password_visible {
                    app.auth_password_textarea.lines().join("\n")
                } else {
                    app.auth_password_textarea
                        .lines()
                        .iter()
                        .map(|line| "•".repeat(line.len()))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                if app.current_detail_field == DetailField::AuthPassword {
                    frame.render_widget(&app.auth_password_textarea, password_area);
                } else {
                    frame.render_widget(
                        Paragraph::new(password_text)
                            .style(Style::default())
                            .wrap(Wrap { trim: true }),
                        password_area,
                    );
                }
            }
        }
        let send_text = if app.is_sending {
            "⏳ Sending Request..."
        } else {
            "🚀 Press Ctrl+S to Send Request"
        };

        let send_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(if app.is_sending {
                Color::Yellow
            } else {
                Color::Green
            }));

        let send_paragraph = Paragraph::new(send_text).block(send_block);
        frame.render_widget(send_paragraph, details_layout[5]);

        // Response Area
        if let Some(response) = &app.last_response {
            let status_color = match response.status {
                200..=299 => Color::Green,
                300..=399 => Color::Blue,
                400..=499 => Color::Yellow,
                _ => Color::Red,
            };

            let response_text = format!(
                "Status: {} {}\nTime: {}ms\n\nHeaders:\n{}\n\nBody:\n{}",
                response.status,
                response.status_text,
                response.time_taken.as_millis(),
                response
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n"),
                response.body
            );

            let response_block = Block::default()
                .borders(Borders::ALL)
                .title("Response")
                .title_style(Style::default().fg(status_color));

            let response_paragraph = Paragraph::new(response_text)
                .block(response_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(response_paragraph, details_layout[6]);
        }
    } else {
        // If no request is selected, show default message centered in the block
        frame.render_widget(
            Paragraph::new("Select a request to view details"),
            inner_area,
        );
    }

    // Footer
    let current_navigation_text = vec![
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
            CurrentScreen::Editing => {
                Span::styled("Editing Mode", Style::default().fg(Color::Yellow))
            }
            CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::LightRed)),
            CurrentScreen::Deleting => Span::styled("Deleting", Style::default().fg(Color::Red)),
            CurrentScreen::DeleteConfirm => {
                Span::styled("Delete Confirmation", Style::default().fg(Color::Red))
            }
            CurrentScreen::AddingRequest => {
                Span::styled("Adding Request", Style::default().fg(Color::Yellow))
            }
            CurrentScreen::RequestDetail => {
                Span::styled("Request Detail", Style::default().fg(Color::Blue))
            }
        },
        Span::styled(" | ", Style::default().fg(Color::White)),
        if app.groups.is_some() {
            Span::styled("Editing Group Name", Style::default().fg(Color::Green))
        } else {
            Span::styled("Not Editing", Style::default().fg(Color::DarkGray))
        },
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = match app.current_screen {
        CurrentScreen::Main => {
            "(q) quit / (e) new group / (a) add request / (↑↓) select group / (→ ←) minimize/maximize group / (→) Details Pane on Request"
        }
        CurrentScreen::Editing => "(ESC) cancel / (Enter) save",
        CurrentScreen::Deleting => "(↑/↓) select group / (Enter) confirm / (ESC) cancel",
        CurrentScreen::DeleteConfirm => "Are you sure you want to delete this group? (y/n)",
        CurrentScreen::Exiting => "Are you sure you want to quit? (y/n)",
        CurrentScreen::AddingRequest => "(ESC) cancel / (Enter) save / (→) change type",
        CurrentScreen::RequestDetail => "(ESC) back / (Tab) next field / (Shift+Tab) previous field",
    };

    let key_notes_footer = Paragraph::new(Line::from(Span::styled(
        current_keys_hint,
        Style::default().fg(Color::Red),
    )))
    .block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(chunks[2]);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);

    // Editing popup
    if let Some(Groups::Name) = app.groups {
        editing_popup(frame, app);
    }

    // TODO - add delete group popup
    // TODO - add delete request popup

    // Exit popup
    if app.current_screen == CurrentScreen::Exiting {
        exiting_popup(frame);
    }

    // Add Request popup
    if app.current_screen == CurrentScreen::AddingRequest {
        add_request_popup(frame, app);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
