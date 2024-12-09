use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::models::AuthType;
use crate::app::models::RequestDetails;
use crate::app::state::App;
use crate::app::ui_state::{ActivePanel, DetailField};

pub fn detail_view_component(
    app: &mut App,
    inner_layout: std::rc::Rc<[Rect]>,
    frame: &mut Frame<'_>,
) -> Rect {
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
    inner_area
}

pub fn render_url_section(frame: &mut Frame, app: &App, area: Rect) {
    let url_block = Block::default()
        .borders(Borders::ALL)
        .title("URL")
        .border_style(if app.current_detail_field == DetailField::Url {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let url_area = url_block.inner(area);
    frame.render_widget(url_block, area);

    if app.current_detail_field == DetailField::Url {
        frame.render_widget(&app.url_textarea, url_area);
    } else {
        frame.render_widget(
            Paragraph::new(app.url_textarea.lines().join("\n")).style(Style::default()),
            url_area,
        );
    }
}

pub fn render_params_section(
    frame: &mut Frame,
    app: &App,
    request_details: &RequestDetails,
    area: Rect,
) {
    let params_text = request_details
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

    frame.render_widget(params, area);
}

pub fn render_headers_section(
    frame: &mut Frame,
    app: &App,
    request_details: &RequestDetails,
    area: Rect,
) {
    let headers_text = request_details
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

    frame.render_widget(headers, area);
}

pub fn render_auth_section(
    frame: &mut Frame,
    app: &App,
    request_details: &RequestDetails,
    area: Rect,
) {
    let auth_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Auth Type
            Constraint::Min(1),    // Auth Details
        ])
        .split(area);

    // Auth Type
    let auth_type_text = format!("Auth Type: {}", request_details.auth_type.as_str());
    let auth_type =
        Paragraph::new(auth_type_text).block(Block::default().borders(Borders::ALL).border_style(
            if app.current_detail_field == DetailField::AuthType {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        ));
    frame.render_widget(auth_type, auth_layout[0]);

    match request_details.auth_type {
        AuthType::None => {
            let no_auth = Paragraph::new("No authentication required")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(no_auth, auth_layout[1]);
        }
        AuthType::Basic => render_basic_auth_section(frame, app, auth_layout[1]),
    }
}

pub fn render_basic_auth_section(frame: &mut Frame, app: &App, area: Rect) {
    let basic_auth_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Username
            Constraint::Length(3), // Password
        ])
        .split(area);

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

pub fn render_send_request_section(frame: &mut Frame, app: &App, area: Rect) {
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
    frame.render_widget(send_paragraph, area);
}

pub fn render_response_section(frame: &mut Frame, app: &App, area: Rect) {
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

        frame.render_widget(response_paragraph, area);
    }
}
