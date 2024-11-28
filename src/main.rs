pub mod app;
pub mod ui;

use app::{App, CurrentScreen, DetailField, Groups, RequestType};
use std::io;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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
                    CurrentScreen::Main => match key.code {
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
                        KeyCode::Right => {
                            if let Some(index) = app.selected_group_index {
                                if index < app.groups_vec.len() {
                                    let group_name = &app.groups_vec[index].clone();
                                    app.toggle_group_minimized(group_name);
                                }
                            }
                        }
                        KeyCode::Left => {
                            if let Some(index) = app.selected_group_index {
                                if index < app.groups_vec.len() {
                                    let group_name = &app.groups_vec[index].clone();
                                    app.minimized_groups.remove(group_name);
                                }
                            }
                        }
                        KeyCode::Up => {
                            app.previous_visible_group();
                        }
                        KeyCode::Down => {
                            app.next_visible_group();
                        }
                        KeyCode::Char('a') => {
                            if !app.list.is_empty() {
                                if let Some(index) = app.selected_group_index {
                                    app.selected_group = Some(app.groups_vec[index].clone());
                                    app.current_screen = CurrentScreen::AddingRequest;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(group_index) = app.selected_group_index {
                                if let Some(group_name) = app.groups_vec.get(group_index) {
                                    if let Some(requests) = app.list.get(group_name) {
                                        if !requests.is_empty() {
                                            // Use the temp_selected_request_index if it exists, otherwise use 0
                                            app.selected_request_index = app.temp_selected_request_index.or(Some(0));
                                            app.current_screen = CurrentScreen::RequestDetail;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Tab => {
                            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.previous_request();
                            } else {
                                app.next_request();
                            }
                        }
                        KeyCode::BackTab => {
                            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.previous_request();
                            } else {
                                app.next_request();
                            }
                        }
                        _ => {}
                    },
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
                            app.save_textarea_content();
                            app.current_screen = CurrentScreen::Main;
                            app.selected_request_index = None;
                            app.current_detail_field = DetailField::None;
                        }
                        KeyCode::Tab => {
                            app.current_detail_field = match app.current_detail_field {
                                DetailField::None => DetailField::Url,
                                DetailField::Url => DetailField::Body,
                                DetailField::Body => DetailField::Headers,
                                DetailField::Headers => DetailField::AuthType,
                                DetailField::AuthType => DetailField::AuthUsername,
                                DetailField::AuthUsername => DetailField::AuthPassword,
                                DetailField::AuthPassword => DetailField::None,
                            };
                        }
                        KeyCode::BackTab => {
                            app.current_detail_field = match app.current_detail_field {
                                DetailField::None => DetailField::AuthPassword,
                                DetailField::Url => DetailField::None,
                                DetailField::Body => DetailField::Url,
                                DetailField::Headers => DetailField::Body,
                                DetailField::AuthType => DetailField::Headers,
                                DetailField::AuthUsername => DetailField::AuthType,
                                DetailField::AuthPassword => DetailField::AuthUsername,
                            };
                        }
                        KeyCode::Left => {
                            if app.current_detail_field == DetailField::AuthType {
                                app.previous_auth_type();
                            } else {
                                match app.current_detail_field {
                                    DetailField::Url => app.url_textarea.input(Event::Key(key)),
                                    DetailField::Body => app.body_textarea.input(Event::Key(key)),
                                    DetailField::AuthUsername => app.auth_username_textarea.input(Event::Key(key)),
                                    DetailField::AuthPassword => app.auth_password_textarea.input(Event::Key(key)),
                                    DetailField::Headers => false, // Headers are not currently editable
                                    DetailField::AuthType => false, // Auth type is only changed via left/right arrows
                                    DetailField::None => false, // No field selected, nothing to do
                                };
                            }
                        }
                        KeyCode::Right => {
                            if app.current_detail_field == DetailField::AuthType {
                                app.next_auth_type();
                            } else {
                                match app.current_detail_field {
                                    DetailField::Url => app.url_textarea.input(Event::Key(key)),
                                    DetailField::Body => app.body_textarea.input(Event::Key(key)),
                                    DetailField::AuthUsername => app.auth_username_textarea.input(Event::Key(key)),
                                    DetailField::AuthPassword => app.auth_password_textarea.input(Event::Key(key)),
                                    DetailField::Headers => false, // Headers are not currently editable
                                    DetailField::AuthType => false, // Auth type is only changed via left/right arrows
                                    DetailField::None => false, // No field selected, nothing to do
                                };
                            }
                        }
                        _ => {
                            match app.current_detail_field {
                                DetailField::Url => app.url_textarea.input(Event::Key(key)),
                                DetailField::Body => app.body_textarea.input(Event::Key(key)),
                                DetailField::AuthUsername => app.auth_username_textarea.input(Event::Key(key)),
                                DetailField::AuthPassword => app.auth_password_textarea.input(Event::Key(key)),
                                DetailField::Headers => false, // Headers are not currently editable
                                DetailField::AuthType => false, // Auth type is only changed via left/right arrows
                                DetailField::None => false, // No field selected, nothing to do
                            };
                        }
                    },
                }
            }
        }
    }
}