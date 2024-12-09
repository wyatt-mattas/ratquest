use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
pub struct RequestResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_taken: Duration,
}
