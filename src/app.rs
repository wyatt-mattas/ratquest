use std::collections::{HashMap, HashSet};


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

// Add a new struct for request details
#[derive(Clone, Debug)]
pub struct RequestDetails {
    pub url: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub auth: Option<String>,
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
    Auth,
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
                auth: None,
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
        // Store the current field type before the mutable borrow
        let current_field = self.current_detail_field.clone();
        
        if let Some(request) = self.get_selected_request_mut() {
            match current_field {
                DetailField::Url => request.details.url.push(c),
                DetailField::Body => request.details.body.push(c),
                DetailField::Headers => {}, // TODO: Implement header editing
                DetailField::Auth => {
                    if let Some(ref mut auth) = request.details.auth {
                        auth.push(c);
                    } else {
                        request.details.auth = Some(String::from(c));
                    }
                },
                DetailField::None => {}
            }
        }
    }

    pub fn pop_from_field(&mut self) {
        // Store the current field type before the mutable borrow
        let current_field = self.current_detail_field.clone();
        
        if let Some(request) = self.get_selected_request_mut() {
            match current_field {
                DetailField::Url => { request.details.url.pop(); }
                DetailField::Body => { request.details.body.pop(); }
                DetailField::Headers => {}, // TODO: Implement header editing
                DetailField::Auth => {
                    if let Some(ref mut auth) = request.details.auth {
                        auth.pop();
                        if auth.is_empty() {
                            request.details.auth = None;
                        }
                    }
                },
                DetailField::None => {}
            }
        }
    }
}