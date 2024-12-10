use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::{App, CurrentScreen, Groups};
use crate::ui::popups::{
    add_request_popup, editing_popup, exiting_popup, render_header_popup, render_params_popup,
};
use crate::ui_components;
use ui_components::details::*;
use ui_components::footer::*;
use ui_components::groups::*;

use super::popups::delete_confirmation_popup;
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

pub fn render_base_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
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

    render_groups(frame, app, &inner_layout);

    // Update the details block:
    let inner_area = detail_view_component(app, inner_layout, frame);

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

        render_url_section(frame, app, details_layout[0]);
        render_body_section(frame, app, details_layout[1]);
        render_params_section(frame, app, &request.details, details_layout[2]);
        render_headers_section(frame, app, &request.details, details_layout[3]);
        render_auth_section(frame, app, &request.details, details_layout[4]);
        render_send_request_section(frame, app, details_layout[5]);
        render_response_section(frame, app, details_layout[6]);
    } else {
        // If no request is selected, show default message centered in the block
        frame.render_widget(
            Paragraph::new("Select a request to view details"),
            inner_area,
        );
    }

    // Footer
    render_footer(frame, app, chunks[2]);

    // Render popups if needed
    if let Some(Groups::Name) = app.groups {
        editing_popup(frame, app);
    }

    if app.current_screen == CurrentScreen::Exiting {
        exiting_popup(frame);
    }

    if app.current_screen == CurrentScreen::AddingRequest {
        add_request_popup(frame, app);
    }

    if app.current_screen == CurrentScreen::DeleteConfirm {
        delete_confirmation_popup(frame, app);
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
