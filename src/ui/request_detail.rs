use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use crate::app::{App, AuthType, DetailField};

// Helper function to render a text area with proper cursor positioning
fn render_textarea(frame: &mut Frame, area: Rect, textarea: &TextArea, title: &str, is_focused: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(textarea, inner_area);
}

fn calculate_content_height(textarea: &TextArea, min_height: u16, max_height: u16) -> u16 {
    let line_count = textarea.lines().len() as u16;
    // Add 2 to account for borders
    let required_height = line_count + 2;
    required_height.clamp(min_height, max_height)
}

pub fn render_request_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    let request = app.get_selected_request().unwrap();

    // Title
    let title = Paragraph::new(Text::styled(
        format!("{} Request: {}", request.request_type.as_str(), request.name),
        Style::default().fg(Color::Green),
    ))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Calculate body height based on content
    let body_height = calculate_content_height(&app.body_textarea, 6, 16);

    // Content with dynamic body height
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                // URL
            Constraint::Length(body_height),      // Body (dynamic)
            Constraint::Length(8),                // Headers
            Constraint::Length(8),                // Auth
        ])
        .split(chunks[1]);

    // URL
    render_textarea(
        frame,
        content_chunks[0],
        &app.url_textarea,
        "URL",
        app.current_detail_field == DetailField::Url
    );

    // Body
    render_textarea(
        frame,
        content_chunks[1],
        &app.body_textarea,
        "Body",
        app.current_detail_field == DetailField::Body
    );

    // Headers
    let headers_text = request.details.headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join("\n");
    let headers = Paragraph::new(headers_text)
        .block(Block::default().borders(Borders::ALL).title("Headers"))
        .style(if app.current_detail_field == DetailField::Headers {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    frame.render_widget(headers, content_chunks[2]);

    // Auth section
    let auth_area = content_chunks[3];
    let auth_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Auth Type Selection
            Constraint::Min(1),     // Auth Details
        ])
        .split(auth_area);

    // Auth Type Selection
    let auth_type_text = format!("Auth Type: {} (←/→ to change)", request.details.auth_type.as_str());
    let auth_type = Paragraph::new(auth_type_text)
        .block(Block::default().borders(Borders::ALL))
        .style(if app.current_detail_field == DetailField::AuthType {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    frame.render_widget(auth_type, auth_chunks[0]);

    // Auth Details
    match request.details.auth_type {
        AuthType::None => {
            let no_auth = Paragraph::new("No authentication required")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(no_auth, auth_chunks[1]);
        }
        AuthType::Basic => {
            let basic_auth_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Username
                    Constraint::Length(3),  // Password
                ])
                .split(auth_chunks[1]);

            // Username
            render_textarea(
                frame,
                basic_auth_chunks[0],
                &app.auth_username_textarea,
                "Username",
                app.current_detail_field == DetailField::AuthUsername
            );

            // Password
            render_textarea(
                frame,
                basic_auth_chunks[1],
                &app.auth_password_textarea,
                "Password",
                app.current_detail_field == DetailField::AuthPassword
            );
        }
    }

    // Footer with keybindings
    let footer = Paragraph::new(
        "ESC: Back | Tab: Next Field | Shift+Tab: Previous Field | ←/→: Change Auth Type"
    )
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}