pub mod routes;
pub mod handlers;
pub mod viewmodels;


use std::sync::Arc;

use crate::application::services::DashboardService;

#[derive(Clone)]
pub struct AppState {
    pub dashboard_service: Arc<DashboardService>,
    pub app_title: String,
    pub app_text_color: String,
}

// Re-export router builder for convenient use in bootstrap
pub use routes::build_router;
