use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, DetailField};

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

    // Content
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // URL
            Constraint::Length(3),  // Body
            Constraint::Length(8),  // Headers
            Constraint::Length(3),  // Auth
        ])
        .split(chunks[1]);

    // URL
    let url_style = if app.current_detail_field == DetailField::Url {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let url = Paragraph::new(request.details.url.as_str())
        .block(Block::default().borders(Borders::ALL).title("URL"))
        .style(url_style);
    frame.render_widget(url, content_chunks[0]);

    // Body
    let body_style = if app.current_detail_field == DetailField::Body {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let body = Paragraph::new(request.details.body.as_str())
        .block(Block::default().borders(Borders::ALL).title("Body"))
        .style(body_style)
        .wrap(Wrap { trim: true });
    frame.render_widget(body, content_chunks[1]);

    // Headers
    let headers_style = if app.current_detail_field == DetailField::Headers {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let headers_text = request.details.headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join("\n");
    let headers = Paragraph::new(headers_text)
        .block(Block::default().borders(Borders::ALL).title("Headers"))
        .style(headers_style);
    frame.render_widget(headers, content_chunks[2]);

    // Auth
    let auth_style = if app.current_detail_field == DetailField::Auth {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let auth = Paragraph::new(request.details.auth.as_deref().unwrap_or(""))
        .block(Block::default().borders(Borders::ALL).title("Auth"))
        .style(auth_style);
    frame.render_widget(auth, content_chunks[3]);

    // Footer with keybindings
    let footer = Paragraph::new("ESC: Back | Tab: Next Field | Shift+Tab: Previous Field")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}