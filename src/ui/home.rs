use tuirealm::{
    props::Alignment,
    ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Style},
        text::{Line, Span, Text},
        widgets::{Block, Borders, Clear, Paragraph, Wrap},
        Frame,
    },
};

use crate::app::{ActivePanel, App, CurrentScreen, DetailField, Groups, RequestType};

pub fn ui(frame: &mut Frame, app: &mut App) {
    render_main_ui(frame, app);
}

fn render_main_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "API Groups",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, chunks[0]);

    // Main body layout
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[1]);

    let tree_block = Block::default()
        .borders(Borders::ALL)
        .title("API Groups")
        .style(if app.active_panel == ActivePanel::Tree {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let tree_area = tree_block.inner(inner_layout[0]);
    frame.render_widget(tree_block, inner_layout[0]);
    app.render_tree_view(frame, tree_area);

    // Update the details block:
    let details_block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .style(if app.active_panel == ActivePanel::Details {
            Style::default()
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
                Constraint::Length(3), // URL
                Constraint::Length(6), // Body
                Constraint::Length(8), // Headers
                Constraint::Length(8), // Auth
            ])
            .split(inner_area);

        // URL Section
        let url_block = Block::default().borders(Borders::ALL).title("URL").style(
            if app.current_detail_field == DetailField::Url {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );

        let url_area = url_block.inner(details_layout[0]);
        frame.render_widget(url_block, details_layout[0]);
        frame.render_widget(
            Paragraph::new(app.url_textarea.lines().join("\n")).style(Style::default()),
            url_area,
        );

        // Body Section
        let body_block = Block::default().borders(Borders::ALL).title("Body").style(
            if app.current_detail_field == DetailField::Body {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );

        let body_area = body_block.inner(details_layout[1]);
        frame.render_widget(body_block, details_layout[1]);
        frame.render_widget(
            Paragraph::new(app.body_textarea.lines().join("\n")).style(Style::default()),
            body_area,
        );

        // Headers Section
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
                .title("Headers")
                .style(if app.current_detail_field == DetailField::Headers {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        );
        frame.render_widget(headers, details_layout[2]);

        // Auth Section
        let auth_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Auth Type
                Constraint::Min(1),    // Auth Details
            ])
            .split(details_layout[3]);

        // Auth Type
        let auth_type_text = format!("Auth Type: {}", request.details.auth_type.as_str());
        let auth_type =
            Paragraph::new(auth_type_text).block(Block::default().borders(Borders::ALL).style(
                if app.current_detail_field == DetailField::AuthType {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            ));
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
                    .style(if app.current_detail_field == DetailField::AuthUsername {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });

                let username_area = username_block.inner(basic_auth_layout[0]);
                frame.render_widget(username_block, basic_auth_layout[0]);
                frame.render_widget(
                    Paragraph::new(app.auth_username_textarea.lines().join("\n"))
                        .style(Style::default()),
                    username_area,
                );

                // Password
                let password_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Password")
                    .style(if app.current_detail_field == DetailField::AuthPassword {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });

                let password_area = password_block.inner(basic_auth_layout[1]);
                frame.render_widget(password_block, basic_auth_layout[1]);
                frame.render_widget(
                    Paragraph::new(app.auth_password_textarea.lines().join("\n"))
                        .style(Style::default()),
                    password_area,
                );
            }
        }
    } else {
        // If no request is selected, show default message centered in the block
        frame.render_widget(
            Paragraph::new("Select a request to view details").alignment(Alignment::Center),
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
        let popup_block = Block::default()
            .title("Enter group name")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup_block, area);

        let input_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let input = Paragraph::new(app.key_input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::White));

        let input_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area)[1];

        frame.render_widget(input, input_area);
    }

    // TODO - add delete group popup
    // TODO - add delete request popup

    // Exit popup
    if app.current_screen == CurrentScreen::Exiting {
        let popup_block = Block::default()
            .title("Quit")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(Clear, area);

        let exit_text = Text::styled(
            "Are you sure you want to quit? (y/n)",
            Style::default().fg(Color::Red),
        );
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        frame.render_widget(exit_paragraph, area);
    }

    // Add Request popup
    if app.current_screen == CurrentScreen::AddingRequest {
        let popup_block = Block::default()
            .title("Add New Request")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup_block, area);

        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Request name
                Constraint::Length(3), // Request type
                Constraint::Length(2), // Instructions
            ])
            .split(area);

        // Request name input
        let name_block = Block::default().title("Request Name").borders(Borders::ALL);
        let name_input = Paragraph::new(app.request_name_input.as_str())
            .block(name_block)
            .style(Style::default().fg(Color::White));
        frame.render_widget(name_input, inner_area[0]);

        // Request type selection
        let type_block = Block::default()
            .title("Request Type (←/→ to change)")
            .borders(Borders::ALL);
        let type_text = Paragraph::new(app.selected_request_type.as_str())
            .block(type_block)
            .style(Style::default().fg(match app.selected_request_type {
                RequestType::GET => Color::Green,
                RequestType::POST => Color::Blue,
                RequestType::PUT => Color::Yellow,
                RequestType::DELETE => Color::Red,
                RequestType::PATCH => Color::Magenta,
            }));
        frame.render_widget(type_text, inner_area[1]);

        // Instructions
        let instructions = Paragraph::new("Press Enter to save, Esc to cancel")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, inner_area[2]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
