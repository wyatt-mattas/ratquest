use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use std::rc::Rc;

use crate::{app::ActivePanel, App};

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

pub fn render_tree(frame: &mut Frame, app: &mut App, chunks: &Rc<[Rect]>) {
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
