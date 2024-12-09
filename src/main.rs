pub mod app;
pub mod ui;
pub mod ui_components;
pub use app::models::{ApiRequest, AuthDetails, AuthType, BasicAuth, RequestDetails, RequestType};
pub use app::state::{App, CurrentScreen};
pub use app::ui_state::DetailField;
use app::ui_state::{ActivePanel, HeaderInputMode};
use app::{state::Groups, ui_state::ParameterInputMode};

use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

use ui::ui;

use std::io::BufWriter;

fn main() -> Result<(), io::Error> {
    // Terminal initialization
    enable_raw_mode()?;
    let stderr = io::stderr();
    let mut writer = BufWriter::new(stderr);
    execute!(writer, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(writer);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let _res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.current_screen {
                    CurrentScreen::Main => {
                        match app.active_panel {
                            ActivePanel::Tree => match key.code {
                                // Tree navigation
                                KeyCode::Down => {
                                    app.tree_next();
                                }
                                KeyCode::Up => {
                                    app.tree_previous();
                                }
                                KeyCode::Right => {
                                    if let Some(selected_id) = app.tree_state.selected() {
                                        if selected_id.starts_with("request-") {
                                            let parts: Vec<&str> =
                                                selected_id.splitn(3, '-').collect();
                                            if parts.len() == 3 {
                                                let group_name = parts[1].to_string();
                                                if let Some(group_idx) = app
                                                    .groups_vec
                                                    .iter()
                                                    .position(|g| g == &group_name)
                                                {
                                                    if let Some(requests) =
                                                        app.list.get(&group_name)
                                                    {
                                                        if let Some(request_idx) = requests
                                                            .iter()
                                                            .position(|r| r.name == parts[2])
                                                        {
                                                            app.selected_group_index =
                                                                Some(group_idx);
                                                            app.selected_request_index =
                                                                Some(request_idx);
                                                            app.sync_textarea_content();
                                                            app.switch_to_details();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        app.tree_toggle();
                                    }
                                }
                                KeyCode::Left => {
                                    app.tree_toggle();
                                }
                                KeyCode::Enter => {
                                    if let Some((group_name, request_idx)) =
                                        app.handle_tree_selection()
                                    {
                                        app.selected_group_index = Some(
                                            app.groups_vec
                                                .iter()
                                                .position(|g| g == &group_name)
                                                .unwrap_or(0),
                                        );
                                        app.selected_request_index = Some(request_idx);
                                        app.switch_to_details();
                                        app.sync_textarea_content();
                                    }
                                }
                                KeyCode::Char('q') => {
                                    app.current_screen = CurrentScreen::Exiting;
                                }
                                KeyCode::Char('e') => {
                                    app.current_screen = CurrentScreen::Editing;
                                    app.groups = Some(Groups::Name);
                                }
                                KeyCode::Char('d') => {
                                    if !app.list.is_empty() {
                                        app.update_groups_vec();
                                        app.current_screen = CurrentScreen::Deleting;
                                    }
                                }
                                KeyCode::Char('a') => {
                                    if !app.list.is_empty() {
                                        if let Some(selected_id) = app.tree_state.selected() {
                                            if selected_id.starts_with("group-") {
                                                let group_name = selected_id
                                                    .strip_prefix("group-")
                                                    .unwrap()
                                                    .to_string();
                                                app.add_request(group_name);
                                            } else if selected_id.starts_with("request-") {
                                                let parts: Vec<&str> =
                                                    selected_id.splitn(3, '-').collect();
                                                if parts.len() == 3 {
                                                    app.add_request(parts[1].to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            },
                            ActivePanel::Details => {
                                match app.current_detail_field {
                                    DetailField::Params => {
                                        if app.adding_params {
                                            match key.code {
                                                KeyCode::Esc => {
                                                    app.adding_params = false;
                                                    app.params_key_input.clear();
                                                    app.params_value_input.clear();
                                                }
                                                KeyCode::Enter => match app.params_input_mode {
                                                    ParameterInputMode::Key => {
                                                        if !app.params_key_input.is_empty() {
                                                            app.toggle_params_input_mode();
                                                        }
                                                    }
                                                    ParameterInputMode::Value => {
                                                        if !app.params_value_input.is_empty() {
                                                            app.save_params();
                                                        }
                                                    }
                                                },
                                                KeyCode::Tab => {
                                                    if !app.params_key_input.is_empty() {
                                                        app.toggle_params_input_mode();
                                                    }
                                                }
                                                KeyCode::Char(c) => match app.params_input_mode {
                                                    ParameterInputMode::Key => {
                                                        app.params_key_input.push(c)
                                                    }
                                                    ParameterInputMode::Value => {
                                                        app.params_value_input.push(c)
                                                    }
                                                },
                                                KeyCode::Backspace => match app.params_input_mode {
                                                    ParameterInputMode::Key => {
                                                        app.params_key_input.pop();
                                                    }
                                                    ParameterInputMode::Value => {
                                                        app.params_value_input.pop();
                                                    }
                                                },
                                                KeyCode::Left
                                                | KeyCode::Right
                                                | KeyCode::Up
                                                | KeyCode::Down => {
                                                    // Ignore navigation keys while adding header
                                                }
                                                _ => {}
                                            }
                                        } else {
                                            match key.code {
                                                KeyCode::Enter => {
                                                    app.start_adding_params();
                                                }
                                                // Handle navigation for headers section when not adding
                                                KeyCode::Left
                                                | KeyCode::Right
                                                | KeyCode::Up
                                                | KeyCode::Down
                                                | KeyCode::Tab
                                                | KeyCode::BackTab
                                                | KeyCode::Esc => {
                                                    // Fall through to main navigation handling
                                                    handle_common_navigation(app, key);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    DetailField::Headers => {
                                        if app.adding_header {
                                            match key.code {
                                                KeyCode::Esc => {
                                                    app.adding_header = false;
                                                    app.header_key_input.clear();
                                                    app.header_value_input.clear();
                                                }
                                                KeyCode::Enter => match app.header_input_mode {
                                                    HeaderInputMode::Key => {
                                                        if !app.header_key_input.is_empty() {
                                                            app.toggle_header_input_mode();
                                                        }
                                                    }
                                                    HeaderInputMode::Value => {
                                                        if !app.header_value_input.is_empty() {
                                                            app.save_header();
                                                        }
                                                    }
                                                },
                                                KeyCode::Tab => {
                                                    if !app.header_key_input.is_empty() {
                                                        app.toggle_header_input_mode();
                                                    }
                                                }
                                                KeyCode::Char(c) => match app.header_input_mode {
                                                    HeaderInputMode::Key => {
                                                        app.header_key_input.push(c)
                                                    }
                                                    HeaderInputMode::Value => {
                                                        app.header_value_input.push(c)
                                                    }
                                                },
                                                KeyCode::Backspace => match app.header_input_mode {
                                                    HeaderInputMode::Key => {
                                                        app.header_key_input.pop();
                                                    }
                                                    HeaderInputMode::Value => {
                                                        app.header_value_input.pop();
                                                    }
                                                },
                                                KeyCode::Left
                                                | KeyCode::Right
                                                | KeyCode::Up
                                                | KeyCode::Down => {
                                                    // Ignore navigation keys while adding header
                                                }
                                                _ => {}
                                            }
                                        } else {
                                            match key.code {
                                                KeyCode::Enter => {
                                                    app.start_adding_header();
                                                }
                                                // Handle navigation for headers section when not adding
                                                KeyCode::Left
                                                | KeyCode::Right
                                                | KeyCode::Up
                                                | KeyCode::Down
                                                | KeyCode::Tab
                                                | KeyCode::BackTab
                                                | KeyCode::Esc => {
                                                    // Fall through to main navigation handling
                                                    handle_common_navigation(app, key);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    // Handle all other detail fields
                                    DetailField::Url
                                    | DetailField::Body
                                    | DetailField::AuthType
                                    | DetailField::AuthUsername
                                    | DetailField::AuthPassword
                                    | DetailField::None => {
                                        handle_common_navigation(app, key);
                                    }
                                }
                                // Save content after each edit if needed
                                app.save_textarea_content();
                            }
                        }
                    }
                    CurrentScreen::Editing => match key.code {
                        KeyCode::Esc => {
                            app.groups = None;
                            app.current_screen = CurrentScreen::Main;
                            app.key_input.clear();
                        }
                        KeyCode::Enter => {
                            if !app.key_input.is_empty() {
                                app.save_group();
                                app.current_screen = CurrentScreen::Main;
                            }
                        }
                        KeyCode::Char(c) => {
                            app.key_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.key_input.pop();
                        }
                        _ => {}
                    },
                    CurrentScreen::Deleting => match key.code {
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        KeyCode::Up => {
                            app.previous_group();
                        }
                        KeyCode::Down => {
                            app.next_group();
                        }
                        KeyCode::Enter => {
                            if !app.groups_vec.is_empty() {
                                app.current_screen = CurrentScreen::DeleteConfirm;
                            }
                        }
                        _ => {}
                    },
                    CurrentScreen::DeleteConfirm => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            app.confirm_delete_selected();
                            app.current_screen = CurrentScreen::Main;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        _ => {}
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            app.current_screen = CurrentScreen::Main;
                        }
                        _ => {}
                    },
                    CurrentScreen::AddingRequest => match key.code {
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.request_name_input.clear();
                            app.selected_group = None;
                            app.selected_request_type = RequestType::GET;
                        }
                        KeyCode::Enter => {
                            app.save_request();
                            app.current_screen = CurrentScreen::Main;
                            app.selected_group = None;
                        }
                        KeyCode::Left => {
                            app.previous_request_type();
                        }
                        KeyCode::Right => {
                            app.next_request_type();
                        }
                        KeyCode::Char(c) => {
                            app.request_name_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.request_name_input.pop();
                        }
                        _ => {}
                    },
                    CurrentScreen::RequestDetail => match key.code {
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.selected_request_index = None;
                            app.current_detail_field = DetailField::None;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn handle_common_navigation(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            if c == 's' && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                // Create a runtime and block on the async operation
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _ = rt.block_on(app.send_request());
            } else {
                let _ = match app.current_detail_field {
                    DetailField::Url => app.url_textarea.input(Event::Key(key)),
                    DetailField::Body => app.body_textarea.input(Event::Key(key)),
                    DetailField::AuthUsername => app.auth_username_textarea.input(Event::Key(key)),
                    DetailField::AuthPassword => app.auth_password_textarea.input(Event::Key(key)),
                    _ => false,
                };
            }
        }
        KeyCode::Left => {
            if app.current_detail_field == DetailField::AuthType
                && app.get_current_request_auth_type() != "None"
            {
                app.previous_auth_type();
            } else {
                app.handle_left_in_textarea(key);
            }
        }
        KeyCode::Right => {
            if app.current_detail_field == DetailField::AuthType {
                app.next_auth_type();
            } else {
                let _ = match app.current_detail_field {
                    DetailField::Url => app.url_textarea.input(Event::Key(key)),
                    DetailField::Body => app.body_textarea.input(Event::Key(key)),
                    DetailField::AuthUsername => app.auth_username_textarea.input(Event::Key(key)),
                    DetailField::AuthPassword => app.auth_password_textarea.input(Event::Key(key)),
                    _ => false,
                };
            }
        }
        KeyCode::Up | KeyCode::BackTab => {
            app.current_detail_field = if app.get_current_request_auth_type() == "None" {
                match app.current_detail_field {
                    DetailField::Url => DetailField::AuthType,
                    DetailField::Body => DetailField::Url,
                    DetailField::Params => DetailField::Body,
                    DetailField::Headers => DetailField::Params,
                    DetailField::AuthType => DetailField::Headers,
                    _ => DetailField::AuthType,
                }
            } else {
                match app.current_detail_field {
                    DetailField::Url => DetailField::AuthPassword,
                    DetailField::Body => DetailField::Url,
                    DetailField::Params => DetailField::Body,
                    DetailField::Headers => DetailField::Params,
                    DetailField::AuthType => DetailField::Headers,
                    DetailField::AuthUsername => DetailField::AuthType,
                    DetailField::AuthPassword => DetailField::AuthUsername,
                    _ => DetailField::AuthPassword,
                }
            };
        }
        KeyCode::Down | KeyCode::Tab => {
            app.current_detail_field = if app.get_current_request_auth_type() == "None" {
                match app.current_detail_field {
                    DetailField::Url => DetailField::Body,
                    DetailField::Body => DetailField::Params,
                    DetailField::Params => DetailField::Headers,
                    DetailField::Headers => DetailField::AuthType,
                    DetailField::AuthType => DetailField::Url,
                    _ => DetailField::Url,
                }
            } else {
                match app.current_detail_field {
                    DetailField::Url => DetailField::Body,
                    DetailField::Body => DetailField::Params,
                    DetailField::Params => DetailField::Headers,
                    DetailField::Headers => DetailField::AuthType,
                    DetailField::AuthType => DetailField::AuthUsername,
                    DetailField::AuthUsername => DetailField::AuthPassword,
                    DetailField::AuthPassword => DetailField::Url,
                    _ => DetailField::Url,
                }
            };
        }
        KeyCode::Esc => {
            app.switch_to_tree();
        }
        _ => {
            let _ = match app.current_detail_field {
                DetailField::Url => app.url_textarea.input(Event::Key(key)),
                DetailField::Body => app.body_textarea.input(Event::Key(key)),
                DetailField::AuthUsername => app.auth_username_textarea.input(Event::Key(key)),
                DetailField::AuthPassword => app.auth_password_textarea.input(Event::Key(key)),
                _ => false,
            };
        }
    }
}
