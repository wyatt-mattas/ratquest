use crate::{app::ui_state::HeaderInputMode, ui::centered_rect, RequestType};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::state::App;
use crate::app::ui_state::ParameterInputMode;

pub fn add_request_popup(frame: &mut Frame, app: &App) {
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

pub fn exiting_popup(frame: &mut Frame) {
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

pub fn editing_popup(frame: &mut Frame, app: &App) {
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

pub fn render_header_popup(frame: &mut Frame, app: &App) {
    // First render a Clear widget over the area where the popup will be
    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(Clear, area);

    // Create and render the popup block
    let popup_block = Block::default()
        .title("Add Header")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(popup_block, area);

    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header key
            Constraint::Length(3), // Header value
            Constraint::Length(2), // Instructions
        ])
        .split(area);

    // Header key input
    let key_block = Block::default()
        .title("Header Key")
        .borders(Borders::ALL)
        .border_style(if matches!(app.header_input_mode, HeaderInputMode::Key) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let key_input = Paragraph::new(app.header_key_input.as_str())
        .block(key_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(key_input, inner_area[0]);

    // Header value input
    let value_block = Block::default()
        .title("Header Value")
        .borders(Borders::ALL)
        .border_style(if matches!(app.header_input_mode, HeaderInputMode::Value) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let value_input = Paragraph::new(app.header_value_input.as_str())
        .block(value_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(value_input, inner_area[1]);

    // Instructions
    let instructions = Paragraph::new(match app.header_input_mode {
        HeaderInputMode::Key => "Enter header key (Enter/Tab to move to value)",
        HeaderInputMode::Value => "Enter header value (Enter to save)",
    })
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, inner_area[2]);
}

pub fn render_params_popup(frame: &mut Frame, app: &mut App) {
    // First render a Clear widget over the area where the popup will be
    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(Clear, area);

    // Create and render the popup block
    let popup_block = Block::default()
        .title("Add Parameter")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(popup_block, area);

    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Params key
            Constraint::Length(3), // Params value
            Constraint::Length(2), // Instructions
        ])
        .split(area);

    // Header key input
    let key_block = Block::default()
        .title("Parameter Key")
        .borders(Borders::ALL)
        .border_style(
            if matches!(app.params_input_mode, ParameterInputMode::Key) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );

    let key_input = Paragraph::new(app.params_key_input.as_str())
        .block(key_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(key_input, inner_area[0]);

    // Header value input
    let value_block = Block::default()
        .title("Parameter Value")
        .borders(Borders::ALL)
        .border_style(
            if matches!(app.params_input_mode, ParameterInputMode::Value) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );

    let value_input = Paragraph::new(app.params_value_input.as_str())
        .block(value_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(value_input, inner_area[1]);

    // Instructions
    let instructions = Paragraph::new(match app.params_input_mode {
        ParameterInputMode::Key => "Enter params key (Enter/Tab to move to value)",
        ParameterInputMode::Value => "Enter params value (Enter to save)",
    })
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, inner_area[2]);
}

pub fn delete_confirmation_popup(frame: &mut Frame, app: &App) {
    let popup_block = Block::default()
        .title("Delete Confirmation")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(Clear, area);

    let confirmation_text = Text::styled(
        app.get_delete_confirmation_message(),
        Style::default().fg(Color::Red),
    );
    let confirmation_paragraph = Paragraph::new(confirmation_text)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(confirmation_paragraph, area);
}