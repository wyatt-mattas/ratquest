pub mod app;
pub mod ui;

// Re-export main types that tests will need
pub use app::{
    ApiRequest, App, AuthDetails, AuthType, BasicAuth, CurrentScreen, DetailField, RequestDetails,
    RequestType,
};
