use std::collections::{HashMap, HashSet};

use base64::{prelude::BASE64_STANDARD, Engine};
use tui_textarea::TextArea;

#[derive(Clone, Debug)]
pub enum AuthType {
    None,
    Basic,
    // We can add more auth types later like:
    // Bearer,
    // OAuth2,
    // etc.
}

impl AuthType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthType::None => "None",
            AuthType::Basic => "Basic",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            AuthType::None => AuthType::Basic,
            AuthType::Basic => AuthType::None,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            AuthType::None => AuthType::Basic,
            AuthType::Basic => AuthType::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub enum AuthDetails {
    None,
    Basic(BasicAuth),
}

#[derive(Clone, Debug)]
pub struct RequestDetails {
    pub url: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub auth_type: AuthType,
    pub auth_details: AuthDetails,
}

impl RequestDetails {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            body: String::new(),
            headers: HashMap::new(),
            auth_type: AuthType::None,
            auth_details: AuthDetails::None,
        }
    }

    pub fn get_basic_auth(&self) -> Option<&BasicAuth> {
        if let AuthDetails::Basic(basic) = &self.auth_details {
            Some(basic)
        } else {
            None
        }
    }

    pub fn get_basic_auth_mut(&mut self) -> Option<&mut BasicAuth> {
        if let AuthDetails::Basic(basic) = &mut self.auth_details {
            Some(basic)
        } else {
            None
        }
    }
}


#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Editing,
    Deleting,
    DeleteConfirm,
    AddingRequest,
    RequestDetail,  // New screen type
    Exiting,
}

#[derive(Clone, Debug)]
pub struct ApiRequest {
    pub name: String,
    pub request_type: RequestType,
    pub details: RequestDetails,  // Add details field
}

pub enum Groups {
    Name,
}

#[derive(Clone, Debug)]
pub enum RequestType {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(PartialEq, Clone)]
pub enum DetailField {
    Url,
    Body,
    Headers,
    AuthType,
    AuthUsername,
    AuthPassword,
    None,
}

impl ApiRequest {
    pub fn new(name: String, request_type: RequestType) -> Self {
        Self {
            name,
            request_type,
            details: RequestDetails {
                url: String::new(),
                body: String::new(),
                headers: HashMap::new(),
                auth_type: AuthType::None,
                auth_details: AuthDetails::None,
            },
        }
    }
}

pub struct App {
    pub key_input: String,
    pub request_name_input: String,
    pub current_screen: CurrentScreen,
    pub list: HashMap<String, Vec<ApiRequest>>,  // Changed from HashSet to HashMap to store requests
    pub groups: Option<Groups>,
    pub selected_index: usize,
    pub groups_vec: Vec<String>,
    pub selected_request_type: RequestType,
    pub selected_group: Option<String>,
    pub minimized_groups: HashSet<String>,  // Track which groups are minimized
    pub selected_group_index: Option<usize>, // Track selected group in main view
    pub selected_request_index: Option<usize>,
    pub current_detail_field: DetailField,  // Track which field is being edited
    pub temp_selected_request_index: Option<usize>, // For highlighting in main view
    pub url_textarea: TextArea<'static>,
    pub body_textarea: TextArea<'static>,
    pub auth_username_textarea: TextArea<'static>,
    pub auth_password_textarea: TextArea<'static>,
}

impl RequestType {
    pub fn next(&self) -> Self {
        match self {
            RequestType::GET => RequestType::POST,
            RequestType::POST => RequestType::PUT,
            RequestType::PUT => RequestType::DELETE,
            RequestType::DELETE => RequestType::PATCH,
            RequestType::PATCH => RequestType::GET,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            RequestType::GET => RequestType::PATCH,
            RequestType::POST => RequestType::GET,
            RequestType::PUT => RequestType::POST,
            RequestType::DELETE => RequestType::PUT,
            RequestType::PATCH => RequestType::DELETE,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::GET => "GET",
            RequestType::POST => "POST",
            RequestType::PUT => "PUT",
            RequestType::DELETE => "DELETE",
            RequestType::PATCH => "PATCH",
        }
    }
}


impl App {
    pub fn new() -> Self {
        App {
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
            url_textarea: TextArea::default(),
            body_textarea: TextArea::default(),
            auth_username_textarea: TextArea::default(),
            auth_password_textarea: TextArea::default(),
        }
    }

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
            let auth_username = request.details.get_basic_auth().map(|auth| auth.username.clone());
            let auth_password = request.details.get_basic_auth().map(|auth| auth.password.clone());

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
                    request.details.headers.insert(
                        "Authorization".to_string(),
                        format!("Basic {}", encoded)
                    );
                } else {
                    // Remove the Authorization header if username is empty
                    request.details.headers.remove("Authorization");
                }
            }
        }
    }

    // Add methods to navigate requests within a group
    pub fn next_request(&mut self) {
        if let Some(group_index) = self.selected_group_index {
            if let Some(group_name) = self.groups_vec.get(group_index) {
                if let Some(requests) = self.list.get(group_name) {
                    if !requests.is_empty() {
                        self.temp_selected_request_index = match self.temp_selected_request_index {
                            None => Some(0),
                            Some(current) if current + 1 < requests.len() => Some(current + 1),
                            _ => Some(0) // Wrap around to the beginning
                        };
                    }
                }
            }
        }
    }

    pub fn previous_request(&mut self) {
        if let Some(group_index) = self.selected_group_index {
            if let Some(group_name) = self.groups_vec.get(group_index) {
                if let Some(requests) = self.list.get(group_name) {
                    if !requests.is_empty() {
                        self.temp_selected_request_index = match self.temp_selected_request_index {
                            None => Some(requests.len() - 1),
                            Some(current) if current > 0 => Some(current - 1),
                            _ => Some(requests.len() - 1) // Wrap around to the end
                        };
                    }
                }
            }
        }
    }

    pub fn save_group(&mut self) {
        if !self.key_input.is_empty() {
            self.list.insert(self.key_input.clone(), Vec::new());
            self.key_input.clear();
            self.groups = None;
            self.update_groups_vec();
        }
    }

    pub fn save_request(&mut self) {
        if let Some(group_name) = &self.selected_group {
            if !self.request_name_input.is_empty() {
                if let Some(requests) = self.list.get_mut(group_name) {
                    requests.push(ApiRequest::new(
                        self.request_name_input.clone(),
                        self.selected_request_type.clone(),
                    ));
                }
                self.request_name_input.clear();
                self.selected_request_type = RequestType::GET;
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
            self.selected_index = self.selected_index.checked_sub(1).unwrap_or(self.groups_vec.len() - 1);
        }
    }

    pub fn next_request_type(&mut self) {
        self.selected_request_type = self.selected_request_type.next();
    }

    pub fn previous_request_type(&mut self) {
        self.selected_request_type = self.selected_request_type.previous();
    }

    pub fn confirm_delete_selected(&mut self) {
        if !self.groups_vec.is_empty() {
            let group_to_delete = self.groups_vec[self.selected_index].clone();
            self.list.remove(&group_to_delete);
            self.update_groups_vec();
            self.selected_index = self.selected_index.min(self.groups_vec.len().saturating_sub(1));
        }
    }

    pub fn delete_group(&mut self) {
        if !self.key_input.is_empty() {
            self.list.remove(&self.key_input);
            self.key_input.clear();
            self.groups = None;
        }
    }

    pub fn toggle_group_minimized(&mut self, group_name: &String) {
        if self.minimized_groups.contains(group_name) {
            self.minimized_groups.remove(group_name);
        } else {
            self.minimized_groups.insert(group_name.to_string());
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

    pub fn previous_visible_group(&mut self) {
        if self.groups_vec.is_empty() {
            self.selected_group_index = None;
            return;
        }

        self.selected_group_index = Some(match self.selected_group_index {
            None => self.groups_vec.len() - 1,
            Some(current) => {
                if current == 0 {
                    self.groups_vec.len() - 1
                } else {
                    current - 1
                }
            }
        });
    }

    pub fn push_to_field(&mut self, c: char) {
        match self.current_detail_field {
            DetailField::Url => {
                self.url_textarea.insert_char(c);
            },
            DetailField::Body => {
                self.body_textarea.insert_char(c);
            },
            DetailField::Headers => {}, // TODO: Implement header editing
            DetailField::AuthUsername => {
                self.auth_username_textarea.insert_char(c);
            },
            DetailField::AuthPassword => {
                self.auth_password_textarea.insert_char(c);
            },
            DetailField::AuthType | DetailField::None => {}
        }
    }
    
    pub fn pop_from_field(&mut self) {
        match self.current_detail_field {
            DetailField::Url => {
                self.url_textarea.delete_char();
            },
            DetailField::Body => {
                self.body_textarea.delete_char();
            },
            DetailField::Headers => {}, // TODO: Implement header editing
            DetailField::AuthUsername => {
                self.auth_username_textarea.delete_char();
            },
            DetailField::AuthPassword => {
                self.auth_password_textarea.delete_char();
            },
            DetailField::AuthType | DetailField::None => {}
        }
    }
}