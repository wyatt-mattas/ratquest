use base64::{prelude::BASE64_STANDARD, Engine};
use crossterm::event::{self, Event};
use rat_tree_view::{TreeState, TreeWidget};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
    Frame,
};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tui_textarea::TextArea;

use super::database::Database;
use super::models::*;
use super::requests::RequestResponse;
use super::ui_state::*;

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Editing,
    Deleting,
    DeleteConfirm,
    AddingRequest,
    RequestDetail,
    Exiting,
}

pub enum Groups {
    Name,
}

pub struct App {
    pub key_input: String,
    pub request_name_input: String,
    pub current_screen: CurrentScreen,
    pub list: HashMap<String, Vec<ApiRequest>>,
    pub groups: Option<Groups>,
    pub selected_index: usize,
    pub groups_vec: Vec<String>,
    pub selected_request_type: RequestType,
    pub selected_group: Option<String>,
    pub minimized_groups: HashSet<String>,
    pub selected_group_index: Option<usize>,
    pub selected_request_index: Option<usize>,
    pub current_detail_field: DetailField,
    pub temp_selected_request_index: Option<usize>,
    pub url_textarea: TextArea<'static>,
    pub body_textarea: TextArea<'static>,
    pub auth_username_textarea: TextArea<'static>,
    pub auth_password_textarea: TextArea<'static>,
    pub tree_state: TreeState,
    pub active_panel: ActivePanel,
    pub password_visible: bool,
    pub header_key_input: String,
    pub header_value_input: String,
    pub adding_header: bool,
    pub header_input_mode: HeaderInputMode,
    pub params_key_input: String,
    pub params_value_input: String,
    pub adding_params: bool,
    pub params_input_mode: ParameterInputMode,
    pub is_sending: bool,
    pub last_response: Option<RequestResponse>,
    pub pending_delete_id: Option<String>,
    pub db: Option<Database>,
}

impl App {
    pub fn new() -> Self {
        let mut url_textarea = TextArea::default();
        url_textarea.set_cursor_line_style(Style::default());

        let mut body_textarea = TextArea::default();
        body_textarea.set_cursor_line_style(Style::default());

        let mut auth_username_textarea = TextArea::default();
        auth_username_textarea.set_cursor_line_style(Style::default());

        let mut auth_password_textarea = TextArea::default();
        auth_password_textarea.set_cursor_line_style(Style::default());

        let db = Database::new("ratquest.db").ok();

        let mut app = Self {
            key_input: String::new(),
            request_name_input: String::new(),
            current_screen: CurrentScreen::Main,
            list: HashMap::new(),
            groups: None,
            selected_index: 0,
            groups_vec: Vec::new(),
            selected_request_type: RequestType::GET,
            selected_group: None,
            minimized_groups: HashSet::new(),
            selected_group_index: None,
            selected_request_index: None,
            current_detail_field: DetailField::None,
            temp_selected_request_index: None,
            url_textarea,
            body_textarea,
            auth_username_textarea,
            auth_password_textarea,
            tree_state: TreeState::default(),
            active_panel: ActivePanel::Tree,
            password_visible: false,
            header_key_input: String::new(),
            header_value_input: String::new(),
            adding_header: false,
            header_input_mode: HeaderInputMode::Key,
            params_key_input: String::new(),
            params_value_input: String::new(),
            adding_params: false,
            params_input_mode: ParameterInputMode::Key,
            is_sending: false,
            last_response: None,
            pending_delete_id: None,
            db,
        };

        let initial_tree = app.build_tree();
        app.tree_state.select(&initial_tree, initial_tree.root());

        // Load initial data from database
        if let Some(db) = &app.db {
            if let Ok(groups) = db.get_all_groups() {
                for (group_id, group_name) in groups {
                    if let Ok(requests) = db.get_requests_for_group(group_id) {
                        app.list.insert(group_name, requests);
                    }
                }
                app.update_groups_vec();
            }
        }

        app
    }

    pub async fn send_request(&mut self) -> Result<(), Box<dyn Error>> {
        // First get all data we need
        let request_data = if let Some(request) = self.get_selected_request() {
            Some((
                request.request_type.clone(),
                request.details.url.clone(),
                request.details.body.clone(),
                request.details.headers.clone(),
                request.details.params.clone(),
            ))
        } else {
            None
        };

        // Then use the data to send the request
        if let Some((request_type, url, body, headers, params)) = request_data {
            self.is_sending = true;

            let client = reqwest::Client::new();

            let mut builder = match request_type {
                RequestType::GET => client.get(&url),
                RequestType::POST => client.post(&url),
                RequestType::PUT => client.put(&url),
                RequestType::DELETE => client.delete(&url),
                RequestType::PATCH => client.patch(&url),
            };

            // Add headers
            for (key, value) in headers {
                builder = builder.header(key, value);
            }

            // Add query parameters
            for (key, value) in params {
                builder = builder.query(&[(key, value)]);
            }

            // Add body for non-GET requests
            if !matches!(request_type, RequestType::GET) {
                builder = builder.body(body);
            }

            let start = std::time::Instant::now();
            let response = builder.send().await?;
            let duration = start.elapsed();

            // Store response
            self.last_response = Some(RequestResponse {
                status: response.status().as_u16(),
                status_text: response.status().to_string(),
                headers: response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect(),
                body: response.text().await?,
                time_taken: duration,
            });

            self.is_sending = false;
        }
        Ok(())
    }

    pub fn start_adding_params(&mut self) {
        self.adding_params = true;
        self.params_key_input.clear();
        self.params_value_input.clear();
        self.params_input_mode = ParameterInputMode::Key;
    }

    pub fn save_params(&mut self) {
        // Clone values first to avoid borrow conflicts
        let key = self.params_key_input.clone();
        let value = self.params_value_input.clone();

        if !key.is_empty() && !value.is_empty() {
            if let Some(request) = self.get_selected_request_mut() {
                request.details.params.insert(key, value);
            }
        }

        self.adding_params = false;
        self.params_key_input.clear();
        self.params_value_input.clear();
    }

    pub fn toggle_params_input_mode(&mut self) {
        self.params_input_mode = match self.params_input_mode {
            ParameterInputMode::Key => ParameterInputMode::Value,
            ParameterInputMode::Value => ParameterInputMode::Key,
        };
    }

    pub fn start_adding_header(&mut self) {
        self.adding_header = true;
        self.header_key_input.clear();
        self.header_value_input.clear();
        self.header_input_mode = HeaderInputMode::Key;
    }

    pub fn save_header(&mut self) {
        // Clone values first to avoid borrow conflicts
        let key = self.header_key_input.clone();
        let value = self.header_value_input.clone();

        if !key.is_empty() && !value.is_empty() {
            // Check authorization outside of the mutable borrow
            if key.to_lowercase() != "authorization" {
                if let Some(request) = self.get_selected_request_mut() {
                    request.details.headers.insert(key, value);
                }
            }
        }

        self.adding_header = false;
        self.header_key_input.clear();
        self.header_value_input.clear();
    }

    pub fn toggle_header_input_mode(&mut self) {
        self.header_input_mode = match self.header_input_mode {
            HeaderInputMode::Key => HeaderInputMode::Value,
            HeaderInputMode::Value => HeaderInputMode::Key,
        };
    }

    /// Checks if the cursor is at the start position (0,0) for the currently active textarea.
    /// Returns true in three cases:
    /// 1. There is no text area for the current field (e.g., Headers)
    /// 2. The cursor is at position (0,0) in the active text area
    /// 3. The current detail field is None
    pub fn is_cursor_at_start(&self) -> bool {
        // Use direct pattern matching to check both the field type and cursor position
        // in a single match expression
        match self.current_detail_field {
            DetailField::Url => self.url_textarea.cursor() == (0, 0),
            DetailField::Body => self.body_textarea.cursor() == (0, 0),
            DetailField::AuthUsername => self.auth_username_textarea.cursor() == (0, 0),
            DetailField::AuthPassword => self.auth_password_textarea.cursor() == (0, 0),
            _ => true, // For Headers and other fields without text areas, consider them always "at start"
        }
    }

    // We can also add a method to handle the left arrow key specifically
    pub fn handle_left_in_textarea(&mut self, key: event::KeyEvent) -> bool {
        // If we're at the start of the text and press left, switch to tree view
        if self.is_cursor_at_start() {
            self.switch_to_tree();
            true
        } else {
            // Otherwise, let the text area handle the key normally
            match self.current_detail_field {
                DetailField::Url => self.url_textarea.input(Event::Key(key)),
                DetailField::Body => self.body_textarea.input(Event::Key(key)),
                DetailField::AuthUsername => self.auth_username_textarea.input(Event::Key(key)),
                DetailField::AuthPassword => self.auth_password_textarea.input(Event::Key(key)),
                _ => false,
            }
        }
    }

    pub fn render_tree_view(&mut self, frame: &mut Frame, area: Rect) {
        let tree = self.build_tree();
        let widget = TreeWidget::new(&tree)
            .block(Block::default())
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("→ ".to_string());

        frame.render_stateful_widget(widget, area, &mut self.tree_state);
    }

    // Handle selecting a request from the tree
    pub fn handle_tree_selection(&mut self) -> Option<(String, usize)> {
        let selected_id = self.tree_state.selected()?;

        // Check if it's a request node
        if selected_id.starts_with("request-") {
            let parts: Vec<&str> = selected_id.splitn(3, '-').collect();
            if parts.len() == 3 {
                let group_name = parts[1].to_string();

                if let Some(requests) = self.list.get(&group_name) {
                    for (idx, request) in requests.iter().enumerate() {
                        if request.name == parts[2] {
                            // Update selected group index
                            self.selected_group_index =
                                self.groups_vec.iter().position(|g| g == &group_name);
                            return Some((group_name, idx));
                        }
                    }
                }
            }
        } else if selected_id.starts_with("group-") {
            let group_name = selected_id.strip_prefix("group-")?.to_string();
            self.selected_group_index = self.groups_vec.iter().position(|g| g == &group_name);
        }
        None
    }

    pub fn switch_to_tree(&mut self) {
        self.active_panel = ActivePanel::Tree;
        self.current_detail_field = DetailField::None;
    }

    pub fn switch_to_details(&mut self) {
        self.active_panel = ActivePanel::Details;
        if self.current_detail_field == DetailField::None {
            self.current_detail_field = DetailField::Url;
        }
    }

    // Add this to handle opening request details when Enter is pressed
    pub fn handle_tree_enter(&mut self) {
        if let Some((group_name, request_idx)) = self.handle_tree_selection() {
            self.selected_group_index = Some(
                self.groups_vec
                    .iter()
                    .position(|g| g == &group_name)
                    .unwrap_or(0),
            );
            self.selected_request_index = Some(request_idx);
            self.current_screen = CurrentScreen::RequestDetail;
        }
    }

    pub fn get_current_request_auth_type(&self) -> String {
        if let (Some(group_idx), Some(request_idx)) =
            (self.selected_group_index, self.selected_request_index)
        {
            if let Some(group_name) = self.groups_vec.get(group_idx) {
                if let Some(requests) = self.list.get(group_name) {
                    if let Some(request) = requests.get(request_idx) {
                        return request.details.auth_type.as_str().to_string();
                    }
                }
            }
        }
        "None".to_string()
    }

    pub fn get_current_request(&self) -> Option<&ApiRequest> {
        // First check if there's a selected request
        if let Some(selected_request) = self.get_selected_request() {
            return Some(selected_request);
        }

        // If no selected request, check the tree selection
        if let Some(selected_id) = self.tree_state.selected() {
            if selected_id.starts_with("request-") {
                let parts: Vec<&str> = selected_id.splitn(3, '-').collect();
                if parts.len() == 3 {
                    let group_name = parts[1].to_string();
                    if let Some(requests) = self.list.get(&group_name) {
                        return requests.iter().find(|r| r.name == parts[2]);
                    }
                }
            }
        }
        None
    }

    fn update_selection_from_tree(&mut self) {
        if let Some(selected_id) = self.tree_state.selected() {
            if selected_id.starts_with("request-") {
                let parts: Vec<&str> = selected_id.splitn(3, '-').collect();
                if parts.len() == 3 {
                    let group_name = parts[1].to_string();
                    if let Some(group_idx) = self.groups_vec.iter().position(|g| g == &group_name) {
                        if let Some(requests) = self.list.get(&group_name) {
                            if let Some(request_idx) =
                                requests.iter().position(|r| r.name == parts[2])
                            {
                                self.selected_group_index = Some(group_idx);
                                self.selected_request_index = Some(request_idx);
                                self.sync_textarea_content();
                            }
                        }
                    }
                }
            } else {
                // If we're on a group node, clear the request selection
                self.selected_request_index = None;
            }
        }
    }

    // Then update the navigation methods to use it
    pub fn tree_next(&mut self) {
        let tree = self.build_tree();
        self.tree_state.move_down(&tree);
        self.update_selection_from_tree();
    }

    pub fn tree_previous(&mut self) {
        let tree = self.build_tree();
        self.tree_state.move_up(&tree);
        self.update_selection_from_tree();
    }

    pub fn tree_toggle(&mut self) {
        let tree = self.build_tree();
        if let Some(id) = self.tree_state.selected() {
            if let Some(node) = tree.root().query(&id.to_string()) {
                if self.tree_state.is_open(node) {
                    self.tree_state.close(&tree, node);
                } else {
                    self.tree_state.open(&tree, node);
                }
            }
        }
    }

    pub fn add_request(&mut self, group_name: String) {
        self.selected_group = Some(group_name);
        self.current_screen = CurrentScreen::AddingRequest;

        // Select the group in the tree
        let tree = self.build_tree();
        if let Some(group_node) = tree
            .root()
            .query(&format!("group-{}", self.selected_group.as_ref().unwrap()))
        {
            self.tree_state.select(&tree, group_node);
            self.tree_state.open(&tree, group_node);
        }
    }

    // combine next_auth and previous_auth
    pub fn next_auth_type(&mut self) {
        if let Some(request) = self.get_selected_request_mut() {
            request.details.auth_type = request.details.auth_type.next();
            match request.details.auth_type {
                AuthType::Basic => {
                    request.details.auth_details = AuthDetails::Basic(BasicAuth {
                        username: String::new(),
                        password: String::new(),
                    });
                }
                AuthType::None => {
                    request.details.auth_details = AuthDetails::None;
                    // Remove Authorization header when switching to None
                    request.details.headers.remove("Authorization");
                }
            }
        }
    }

    pub fn previous_auth_type(&mut self) {
        if let Some(request) = self.get_selected_request_mut() {
            request.details.auth_type = request.details.auth_type.previous();
            match request.details.auth_type {
                AuthType::Basic => {
                    request.details.auth_details = AuthDetails::Basic(BasicAuth {
                        username: String::new(),
                        password: String::new(),
                    });
                }
                AuthType::None => {
                    request.details.auth_details = AuthDetails::None;
                    // Remove Authorization header when switching to None
                    request.details.headers.remove("Authorization");
                }
            }
        }
    }

    pub fn sync_textarea_content(&mut self) {
        if let Some(request) = self.get_selected_request() {
            let url = request.details.url.clone();
            let body = request.details.body.clone();
            let auth_username = request
                .details
                .get_basic_auth()
                .map(|auth| auth.username.clone());
            let auth_password = request
                .details
                .get_basic_auth()
                .map(|auth| auth.password.clone());

            self.url_textarea = TextArea::from(vec![url]);
            self.body_textarea = TextArea::from(vec![body]);

            if let Some(username) = auth_username {
                self.auth_username_textarea = TextArea::from(vec![username]);
            } else {
                self.auth_username_textarea = TextArea::default();
            }

            if let Some(password) = auth_password {
                self.auth_password_textarea = TextArea::from(vec![password]);
            } else {
                self.auth_password_textarea = TextArea::default();
            }
        }
    }

    pub fn save_textarea_content(&mut self) {
        // Get all text values first to avoid borrowing conflicts
        let url = self.url_textarea.lines()[0].to_string();
        let body = self.body_textarea.lines()[0].to_string();
        let username = self.auth_username_textarea.lines()[0].to_string();
        let password = self.auth_password_textarea.lines()[0].to_string();

        if let Some(request) = self.get_selected_request_mut() {
            request.details.url = url;
            request.details.body = body;

            if let Some(basic_auth) = request.details.get_basic_auth_mut() {
                basic_auth.username = username.clone();
                basic_auth.password = password.clone();

                // Update the Authorization header for Basic Auth
                if !username.is_empty() {
                    let auth_string = format!("{}:{}", username, password);
                    let encoded = BASE64_STANDARD.encode(auth_string.as_bytes());
                    request
                        .details
                        .headers
                        .insert("Authorization".to_string(), format!("Basic {}", encoded));
                } else {
                    // Remove the Authorization header if username is empty
                    request.details.headers.remove("Authorization");
                }
            }
        }
    }

    pub fn save_group(&mut self) {
        if !self.key_input.is_empty() {
            if let Some(db) = &self.db {
                if let Ok(_) = db.create_group(&self.key_input) {
                    self.list.insert(self.key_input.clone(), Vec::new());
                    self.update_groups_vec();
                }
            }
            self.key_input.clear();
            self.groups = None;
        }
    }

    pub fn save_request(&mut self) {
        if let Some(group_name) = &self.selected_group {
            if !self.request_name_input.is_empty() {
                if let Some(requests) = self.list.get_mut(group_name) {
                    // Create new request with empty details
                    let mut new_request = ApiRequest::new(
                        self.request_name_input.clone(),
                        self.selected_request_type.clone(),
                    );

                    // Initialize empty text areas
                    self.url_textarea = TextArea::default();
                    self.body_textarea = TextArea::default();
                    self.auth_username_textarea = TextArea::default();
                    self.auth_password_textarea = TextArea::default();

                    // Save the empty text areas to the request
                    new_request.details.url = String::new();
                    new_request.details.body = String::new();

                    // Add the new request
                    requests.push(new_request);

                    // After adding the request, update the tree state
                    let tree = self.build_tree();

                    // First, find and open the parent group
                    if let Some(parent) = tree.root().query(&format!("group-{}", group_name)) {
                        // Select the parent group and open it
                        self.tree_state.select(&tree, parent);
                        self.tree_state.open(&tree, parent);

                        // Then try to find and select the newly added request
                        if let Some(request_node) = tree.root().query(&format!(
                            "request-{}-{}",
                            group_name, self.request_name_input
                        )) {
                            self.tree_state.select(&tree, request_node);
                        }
                    }

                    self.request_name_input.clear();
                    self.selected_request_type = RequestType::GET;
                }
            }
        }
    }

    pub fn get_selected_request(&self) -> Option<&ApiRequest> {
        if let Some(group_index) = self.selected_group_index {
            if let Some(group_name) = self.groups_vec.get(group_index) {
                if let Some(requests) = self.list.get(group_name) {
                    if let Some(request_index) = self.selected_request_index {
                        return requests.get(request_index);
                    }
                }
            }
        }
        None
    }

    pub fn get_selected_request_mut(&mut self) -> Option<&mut ApiRequest> {
        if let Some(group_index) = self.selected_group_index {
            if let Some(group_name) = self.groups_vec.get(group_index) {
                if let Some(requests) = self.list.get_mut(group_name) {
                    if let Some(request_index) = self.selected_request_index {
                        return requests.get_mut(request_index);
                    }
                }
            }
        }
        None
    }

    pub fn update_groups_vec(&mut self) {
        self.groups_vec = self.list.keys().cloned().collect();
        self.groups_vec.sort();
    }

    pub fn next_group(&mut self) {
        if !self.groups_vec.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.groups_vec.len();
        }
    }

    pub fn previous_group(&mut self) {
        if !self.groups_vec.is_empty() {
            self.selected_index = self
                .selected_index
                .checked_sub(1)
                .unwrap_or(self.groups_vec.len() - 1);
        }
    }

    pub fn next_request_type(&mut self) {
        self.selected_request_type = self.selected_request_type.next();
    }

    pub fn previous_request_type(&mut self) {
        self.selected_request_type = self.selected_request_type.previous();
    }

    pub fn start_delete_confirmation(&mut self) {
        if let Some(selected_id) = self.tree_state.selected() {
            self.current_screen = CurrentScreen::DeleteConfirm;
            // Store the ID to be deleted so we can reference it in the confirmation
            self.pending_delete_id = Some(selected_id.to_string());
        }
    }

    pub fn confirm_delete_selected(&mut self) {
        if let Some(id) = &self.pending_delete_id {
            if id.starts_with("request-") {
                self.delete_selected_request();
            } else if id.starts_with("group-") {
                self.delete_selected_group();
            }
        }
        self.pending_delete_id = None;
    }

    pub fn get_delete_confirmation_message(&self) -> String {
        if let Some(id) = &self.pending_delete_id {
            if id.starts_with("request-") {
                let parts: Vec<&str> = id.splitn(3, '-').collect();
                if parts.len() == 3 {
                    return format!(
                        "Are you sure you want to delete request '{}'? (y/n)",
                        parts[2]
                    );
                }
            } else if id.starts_with("group-") {
                let group_name = id.strip_prefix("group-").unwrap();
                return format!(
                    "Are you sure you want to delete group '{}' and all its requests? (y/n)",
                    group_name
                );
            }
        }
        "Are you sure you want to delete this item? (y/n)".to_string()
    }

    // Previous delete methods remain the same
    pub fn delete_selected_request(&mut self) {
        if let Some(selected_id) = &self.pending_delete_id {
            if selected_id.starts_with("request-") {
                let parts: Vec<&str> = selected_id.splitn(3, '-').collect();
                if parts.len() == 3 {
                    let group_name = parts[1].to_string();
                    let request_name = parts[2].to_string();

                    if let Some(requests) = self.list.get_mut(&group_name) {
                        if let Some(pos) = requests.iter().position(|r| r.name == request_name) {
                            requests.remove(pos);

                            // Update tree state after deletion
                            let tree = self.build_tree();
                            if let Some(group_node) =
                                tree.root().query(&format!("group-{}", group_name))
                            {
                                self.tree_state.select(&tree, group_node);
                            }

                            // Clear request selection
                            self.selected_request_index = None;
                        }
                    }
                }
            }
        }
    }

    pub fn delete_selected_group(&mut self) {
        if let Some(selected_id) = &self.pending_delete_id {
            if selected_id.starts_with("group-") {
                let group_name = selected_id.strip_prefix("group-").unwrap().to_string();
                self.list.remove(&group_name);
                self.update_groups_vec();

                // Update tree state after deletion
                let tree = self.build_tree();
                if let Some(root) = tree.root().query(&"/".to_string()) {
                    self.tree_state.select(&tree, root);
                }

                // Clear selections
                self.selected_group_index = None;
                self.selected_request_index = None;
            }
        }
    }

    pub fn delete_group(&mut self) {
        if !self.key_input.is_empty() {
            self.list.remove(&self.key_input);
            self.key_input.clear();
            self.groups = None;
        }
    }

    pub fn next_visible_group(&mut self) {
        if self.groups_vec.is_empty() {
            self.selected_group_index = None;
            return;
        }

        self.selected_group_index = Some(match self.selected_group_index {
            None => 0,
            Some(current) => (current + 1) % self.groups_vec.len(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_type_transitions() {
        let request_type = RequestType::GET;

        assert!(matches!(request_type.next(), RequestType::POST));
        assert!(matches!(RequestType::POST.next(), RequestType::PUT));
        assert!(matches!(RequestType::PUT.next(), RequestType::DELETE));
        assert!(matches!(RequestType::DELETE.next(), RequestType::PATCH));
        assert!(matches!(RequestType::PATCH.next(), RequestType::GET));
    }

    #[test]
    fn test_auth_type_transitions() {
        let auth_type = AuthType::None;
        assert!(matches!(auth_type.next(), AuthType::Basic));
        assert!(matches!(AuthType::Basic.next(), AuthType::None));
    }

    #[test]
    fn test_basic_auth_creation() {
        let basic_auth = BasicAuth {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
        };

        assert_eq!(basic_auth.username, "test_user");
        assert_eq!(basic_auth.password, "test_pass");
    }

    #[test]
    fn test_request_details_new() {
        let details = RequestDetails::new();
        assert!(details.url.is_empty());
        assert!(details.body.is_empty());
        assert!(details.headers.is_empty());
        assert!(matches!(details.auth_type, AuthType::None));
    }
}
