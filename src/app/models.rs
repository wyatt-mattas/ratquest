use rat_tree_view::NodeValue;
use ratatui::style::{Color, Style};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ApiRequest {
    pub name: String,
    pub request_type: RequestType,
    pub details: RequestDetails,
}

#[derive(Clone, Debug)]
pub enum RequestType {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Clone, Debug)]
pub enum AuthType {
    None,
    Basic,
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
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub auth_type: AuthType,
    pub auth_details: AuthDetails,
}

impl ApiRequest {
    pub fn new(name: String, request_type: RequestType) -> Self {
        Self {
            name,
            request_type,
            details: RequestDetails {
                url: String::new(),
                body: String::new(),
                params: HashMap::new(),
                headers: HashMap::new(),
                auth_type: AuthType::None,
                auth_details: AuthDetails::None,
            },
        }
    }
}

impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            request_type: RequestType::GET,
            details: RequestDetails {
                url: String::new(),
                body: String::new(),
                params: HashMap::new(),
                headers: HashMap::new(),
                auth_type: AuthType::None,
                auth_details: AuthDetails::None,
            },
        }
    }
}

impl NodeValue for ApiRequest {
    fn render_parts_iter(&self) -> impl Iterator<Item = (&str, Option<Style>)> {
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

impl RequestDetails {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            body: String::new(),
            params: HashMap::new(),
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
