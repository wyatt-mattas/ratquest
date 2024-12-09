pub mod app;
pub mod ui;
pub mod ui_components; // Add this line

// Re-export main types that tests will need
pub use app::models::{ApiRequest, AuthDetails, AuthType, BasicAuth, RequestDetails, RequestType};
pub use app::state::{App, CurrentScreen};
pub use app::ui_state::DetailField;
