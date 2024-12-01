pub mod app;
pub mod ui;

use app::{ActivePanel, App, CurrentScreen, DetailField, Groups, RequestType};
use std::io;
use tuirealm::ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tuirealm::ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use ui::ui;

fn main() -> Result<(), io::Error> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
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
                                // TODO check if AuthType is not None, then only allow switching to other auth types using the left and right arrow keys otherwise allow switching to Main panel
                                match key.code {
                                    KeyCode::Left => {
                                        if app.current_detail_field == DetailField::AuthType
                                            && app.get_current_request_auth_type() != "None"
                                        {
                                            app.previous_auth_type();
                                        } else {
                                            app.switch_to_tree();
                                        }
                                        if app.current_detail_field == DetailField::None
                                            || app.current_detail_field == DetailField::Url
                                            || app.current_detail_field == DetailField::AuthUsername
                                            || app.current_detail_field == DetailField::AuthPassword
                                            || app.current_detail_field == DetailField::Headers
                                            || app.current_detail_field == DetailField::Body
                                        {
                                            app.switch_to_tree();
                                        } else {
                                            let _ = match app.current_detail_field {
                                                DetailField::Url => {
                                                    app.url_textarea.input(Event::Key(key))
                                                }
                                                DetailField::Body => {
                                                    app.body_textarea.input(Event::Key(key))
                                                }
                                                DetailField::AuthUsername => app
                                                    .auth_username_textarea
                                                    .input(Event::Key(key)),
                                                DetailField::AuthPassword => app
                                                    .auth_password_textarea
                                                    .input(Event::Key(key)),
                                                _ => false,
                                            };
                                        }
                                    }
                                    KeyCode::Right => {
                                        if app.current_detail_field == DetailField::AuthType {
                                            app.next_auth_type();
                                        } else {
                                            let _ = match app.current_detail_field {
                                                DetailField::Url => {
                                                    app.url_textarea.input(Event::Key(key))
                                                }
                                                DetailField::Body => {
                                                    app.body_textarea.input(Event::Key(key))
                                                }
                                                DetailField::AuthUsername => app
                                                    .auth_username_textarea
                                                    .input(Event::Key(key)),
                                                DetailField::AuthPassword => app
                                                    .auth_password_textarea
                                                    .input(Event::Key(key)),
                                                _ => false,
                                            };
                                        }
                                    }
                                    // have up and down arrow keys to navigate through the textarea
                                    KeyCode::Up | KeyCode::BackTab => {
                                        app.current_detail_field = if app
                                            .get_current_request_auth_type()
                                            == "None"
                                        {
                                            match app.current_detail_field {
                                                DetailField::Url => DetailField::AuthType,
                                                DetailField::Body => DetailField::Url,
                                                DetailField::Headers => DetailField::Body,
                                                DetailField::AuthType => DetailField::Headers,
                                                _ => DetailField::AuthType,
                                            }
                                        } else {
                                            match app.current_detail_field {
                                                DetailField::Url => DetailField::AuthPassword,
                                                DetailField::Body => DetailField::Url,
                                                DetailField::Headers => DetailField::Body,
                                                DetailField::AuthType => DetailField::Headers,
                                                DetailField::AuthUsername => DetailField::AuthType,
                                                DetailField::AuthPassword => {
                                                    DetailField::AuthUsername
                                                }
                                                _ => DetailField::AuthPassword,
                                            }
                                        };
                                    }
                                    KeyCode::Down | KeyCode::Tab => {
                                        app.current_detail_field = if app
                                            .get_current_request_auth_type()
                                            == "None"
                                        {
                                            match app.current_detail_field {
                                                DetailField::Url => DetailField::Body,
                                                DetailField::Body => DetailField::Headers,
                                                DetailField::Headers => DetailField::AuthType,
                                                DetailField::AuthType => DetailField::Url,
                                                _ => DetailField::Url,
                                            }
                                        } else {
                                            match app.current_detail_field {
                                                DetailField::Url => DetailField::Body,
                                                DetailField::Body => DetailField::Headers,
                                                DetailField::Headers => DetailField::AuthType,
                                                DetailField::AuthType => DetailField::AuthUsername,
                                                DetailField::AuthUsername => {
                                                    DetailField::AuthPassword
                                                }
                                                DetailField::AuthPassword => DetailField::Url,
                                                _ => DetailField::Url,
                                            }
                                        };
                                    }
                                    KeyCode::Char('w') => {
                                        if key.modifiers.contains(event::KeyModifiers::CONTROL)
                                            && app.current_detail_field == DetailField::AuthPassword
                                        {
                                            app.password_visible = !app.password_visible;
                                        } else if app.current_detail_field
                                            == DetailField::AuthPassword
                                        {
                                            let _ =
                                                app.auth_password_textarea.input(Event::Key(key));
                                            app.save_textarea_content();
                                        }
                                    }
                                    KeyCode::Esc => {
                                        app.switch_to_tree();
                                    }
                                    _ => {
                                        let _ = match app.current_detail_field {
                                            DetailField::Url => {
                                                app.url_textarea.input(Event::Key(key))
                                            }
                                            DetailField::Body => {
                                                app.body_textarea.input(Event::Key(key))
                                            }
                                            DetailField::AuthUsername => {
                                                app.auth_username_textarea.input(Event::Key(key))
                                            }
                                            DetailField::AuthPassword => {
                                                app.auth_password_textarea.input(Event::Key(key))
                                            }
                                            _ => false,
                                        };
                                    }
                                }
                                // Save content after each edit
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
