use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use std::rc::Rc;

use crate::app::state::App;
use crate::app::ui_state::{ActivePanel, DetailField};

pub fn title_block_component(frame: &mut Frame, chunks: &Rc<[Rect]>) {
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

pub fn render_groups(frame: &mut Frame, app: &mut App, chunks: &Rc<[Rect]>) {
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

pub fn render_body_section(frame: &mut Frame, app: &App, area: Rect) {
    let body_block = Block::default()
        .borders(Borders::ALL)
        .title("Body")
        .border_style(if app.current_detail_field == DetailField::Body {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let body_area = body_block.inner(area);
    frame.render_widget(body_block, area);

    if app.current_detail_field == DetailField::Body {
        frame.render_widget(&app.body_textarea, body_area);
    } else {
        frame.render_widget(
            Paragraph::new(app.body_textarea.lines().join("\n")).style(Style::default()),
            body_area,
        );
    }
}
