use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::{App, CurrentScreen};

pub fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
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
        CurrentScreen::Main => "(q) quit / (e) new group / (a) add request / (d) delete / (↑↓) select / (→ ←) minimize/maximize / (→) Details",
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
        .split(area);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);
}
