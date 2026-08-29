use async_trait::async_trait;

use crate::{
    domain::{DashboardState, EntityId},
    shared::error::AppResult,
};

#[async_trait]
pub trait HomeAssistantClient: Send + Sync {
    async fn fetch_dashboard_state(&self) -> AppResult<DashboardState>;
    async fn toggle(&self, entity_id: &EntityId) -> AppResult<()>;
    async fn set_brightness(&self, entity_id: &EntityId, brightness_pct: u8) -> AppResult<()>;
    async fn set_color_temp(&self, entity_id: &EntityId, color_temp_kelvin: u16) -> AppResult<()>;
    async fn set_color(&self, entity_id: &EntityId, rgb: [u8; 3]) -> AppResult<()>;
    async fn run_script(&self, entity_id: &EntityId) -> AppResult<()>;
}
