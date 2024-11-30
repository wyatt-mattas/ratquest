use tuirealm::ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentScreen, Groups, RequestType};

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

    // Render the tree view in the left panel
    app.render_tree_view(frame, inner_layout[0]);

    // Render the details view in the right panel
    frame.render_widget(
        Paragraph::new("Details View")
            .block(Block::default().borders(Borders::ALL).title("Details")),
        inner_layout[1],
    );

    frame.render_widget(
        Paragraph::new("inner 1").block(Block::new().borders(Borders::ALL)),
        inner_layout[1],
    );

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
            "(q) quit / (e) new group / (d) delete group / (a) add request / (↑↓) select group / (→←) minimize/maximize / (Tab/Shift+Tab) select request / (Enter) open request"
        }
        CurrentScreen::Editing => {
            "(ESC) cancel / (Enter) save"
        }
        CurrentScreen::Deleting => {
            "(↑/↓) select group / (Enter) confirm / (ESC) cancel"
        }
        CurrentScreen::DeleteConfirm => {
            "Are you sure you want to delete this group? (y/n)"
        }
        CurrentScreen::Exiting => {
            "Are you sure you want to quit? (y/n)"
        }
        CurrentScreen::AddingRequest => {
            "(ESC) cancel / (Enter) save / (←/→) change type"
        }
        CurrentScreen::RequestDetail => {
            "(ESC) back / (Tab) next field / (Shift+Tab) previous field"
        }
    };

    let key_notes_footer = Paragraph::new(Line::from(Span::styled(
        current_keys_hint,
        Style::default().fg(Color::Red),
    )))
    .block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
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

    // // Delete selection popup
    // if app.current_screen == CurrentScreen::Deleting {
    //     let popup_block = Block::default()
    //         .title("Select Group to Delete")
    //         .borders(Borders::ALL)
    //         .style(Style::default().bg(Color::DarkGray));

    //     let area = centered_rect(60, 25, frame.area());
    //     frame.render_widget(Clear, area);

    //     let mut items: Vec<ListItem> = Vec::new();
    //     for (i, group_name) in app.groups_vec.iter().enumerate() {
    //         let style = if i == app.selected_index {
    //             Style::default().fg(Color::Black).bg(Color::White)
    //         } else {
    //             Style::default().fg(Color::White)
    //         };
    //         items.push(ListItem::new(Line::from(Span::styled(group_name, style))));
    //     }

    //     let list = List::new(items).block(popup_block);
    //     frame.render_widget(list, area);
    // }

    // // Delete confirmation popup
    // if app.current_screen == CurrentScreen::DeleteConfirm {
    //     let popup_block = Block::default()
    //         .title("Confirm Deletion")
    //         .borders(Borders::ALL)
    //         .style(Style::default().bg(Color::DarkGray));

    //     let area = centered_rect(60, 25, frame.area());
    //     frame.render_widget(Clear, area);

    //     let selected_group = &app.groups_vec[app.selected_index];
    //     let text = Text::styled(
    //         format!(
    //             "Are you sure you want to delete '{}'? (y/n)",
    //             selected_group
    //         ),
    //         Style::default().fg(Color::Red),
    //     );
    //     let paragraph = Paragraph::new(text)
    //         .block(popup_block)
    //         .wrap(Wrap { trim: false });

    //     frame.render_widget(paragraph, area);
    // }

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
