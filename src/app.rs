use std::collections::{HashMap, HashSet};

use base64::{prelude::BASE64_STANDARD, Engine};
// use std::collections::HashMap;
use tui_realm_treeview::{Node, NodeValue, Tree, TreeState, TreeWidget};
use tui_textarea::TextArea;
// use tuirealm::props::Alignment;
use tuirealm::ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
};

#[derive(PartialEq)]
pub enum ActivePanel {
    Tree,
    Details,
}

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
    RequestDetail, // New screen type
    Exiting,
}

#[derive(Clone, Debug)]
pub struct ApiRequest {
    pub name: String,
    pub request_type: RequestType,
    pub details: RequestDetails, // Add details field
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

// First implement Default for ApiRequest
impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            request_type: RequestType::GET,
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

impl NodeValue for ApiRequest {
    fn render_parts_iter(&self) -> impl Iterator<Item = (&str, Option<Style>)> {
        // Create a TextSpan for the request
        let request_style = Style::default().fg(match self.request_type {
            RequestType::GET => Color::Green,
            RequestType::POST => Color::Blue,
            RequestType::PUT => Color::Yellow,
            RequestType::DELETE => Color::Red,
            RequestType::PATCH => Color::Magenta,
        });

        std::iter::once((&self.name[..], Some(request_style)))
    }
}

#[derive(Default, Clone)]
pub struct TreeNode {
    text: String,
    style: Option<Style>,
}

impl TreeNode {
    fn new(text: String) -> Self {
        Self { text, style: None }
    }

    fn with_style(text: String, style: Style) -> Self {
        Self {
            text,
            style: Some(style),
        }
    }
}

impl NodeValue for TreeNode {
    fn render_parts_iter(&self) -> impl Iterator<Item = (&str, Option<Style>)> {
        vec![(self.text.as_str(), self.style)].into_iter()
    }
}

pub struct App {
    pub key_input: String,
    pub request_name_input: String,
    pub current_screen: CurrentScreen,
    pub list: HashMap<String, Vec<ApiRequest>>, // Changed from HashSet to HashMap to store requests
    pub groups: Option<Groups>,
    pub selected_index: usize,
    pub groups_vec: Vec<String>,
    pub selected_request_type: RequestType,
    pub selected_group: Option<String>,
    pub minimized_groups: HashSet<String>, // Track which groups are minimized
    pub selected_group_index: Option<usize>, // Track selected group in main view
    pub selected_request_index: Option<usize>,
    pub current_detail_field: DetailField, // Track which field is being edited
    pub temp_selected_request_index: Option<usize>, // For highlighting in main view
    pub url_textarea: TextArea<'static>,
    pub body_textarea: TextArea<'static>,
    pub auth_username_textarea: TextArea<'static>,
    pub auth_password_textarea: TextArea<'static>,
    pub tree_state: TreeState,
    pub active_panel: ActivePanel,
    pub password_visible: bool,
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
            url_textarea: TextArea::default(),
            body_textarea: TextArea::default(),
            auth_username_textarea: TextArea::default(),
            auth_password_textarea: TextArea::default(),
            tree_state: TreeState::default(),
            active_panel: ActivePanel::Tree,
            password_visible: false,
        };

        app.tree_state
            .select(app.build_tree().root(), app.build_tree().root());
        app
    }

    pub fn build_tree(&self) -> Tree<TreeNode> {
        let mut root = Node::new("/".to_string(), TreeNode::new("API Groups".to_string()));

        // Add each group as a child of the root
        for (group_name, requests) in &self.list {
            let mut group_node = Node::new(
                format!("group-{}", group_name),
                TreeNode::new(group_name.clone()),
            );

            // Add requests as children of the group
            for request in requests {
                // Create a unique ID for the request
                let request_id = format!("request-{}-{}", group_name, request.name);

                // Create the display text with color formatting
                let (symbol, style) = match request.request_type {
                    RequestType::GET => ("○", Style::default().fg(Color::Green)),
                    RequestType::POST => ("+", Style::default().fg(Color::Blue)),
                    RequestType::PUT => ("↺", Style::default().fg(Color::Yellow)),
                    RequestType::DELETE => ("-", Style::default().fg(Color::Red)),
                    RequestType::PATCH => ("~", Style::default().fg(Color::Magenta)),
                };

                let display_text = format!(
                    "{} {} {}",
                    symbol,
                    request.request_type.as_str(),
                    request.name
                );
                let request_node = Node::new(request_id, TreeNode::with_style(display_text, style));
                group_node.add_child(request_node);
            }

            root.add_child(group_node);
        }

        Tree::new(root)
    }

    pub fn render_tree_view(&mut self, frame: &mut Frame, area: Rect) {
        let tree = self.build_tree();

        let tree_widget = TreeWidget::new(&tree)
            // .block(Block::default().borders(Borders::ALL).title("API Groups"))
            .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
            .style(Style::default().fg(Color::White))
            .indent_size(2)
            .highlight_symbol("→ ".to_string());

        frame.render_stateful_widget(tree_widget, area, &mut self.tree_state);
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
        self.tree_state.move_down(tree.root());
        self.update_selection_from_tree();
    }

    pub fn tree_previous(&mut self) {
        let tree = self.build_tree();
        self.tree_state.move_up(tree.root());
        self.update_selection_from_tree();
    }

    pub fn tree_toggle(&mut self) {
        let tree = self.build_tree();
        if let Some(id) = self.tree_state.selected() {
            if let Some(node) = tree.root().query(&id.to_string()) {
                if self.tree_state.is_open(node) {
                    self.tree_state.close(tree.root());
                } else {
                    self.tree_state.open(tree.root());
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
            self.tree_state.select(tree.root(), group_node);
            self.tree_state.open(tree.root());
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
                        self.tree_state.select(tree.root(), parent);
                        self.tree_state.open(tree.root());

                        // Then try to find and select the newly added request
                        if let Some(request_node) = tree.root().query(&format!(
                            "request-{}-{}",
                            group_name, self.request_name_input
                        )) {
                            self.tree_state.select(tree.root(), request_node);
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

    pub fn confirm_delete_selected(&mut self) {
        if !self.groups_vec.is_empty() {
            let group_to_delete = self.groups_vec[self.selected_index].clone();
            self.list.remove(&group_to_delete);
            self.update_groups_vec();
            self.selected_index = self
                .selected_index
                .min(self.groups_vec.len().saturating_sub(1));
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
