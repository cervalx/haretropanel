use serde::{Deserialize, Serialize};

use super::value_objects::EntityId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityKind {
    Light,
    Switch,
    Sensor,
    Climate,
    Script,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub kind: EntityKind,
    pub is_on: bool,
    pub value: Option<String>,
    pub brightness: Option<u8>,
    pub color_temp_kelvin: Option<u16>,
    pub min_color_temp_kelvin: Option<u16>,
    pub max_color_temp_kelvin: Option<u16>,
    pub rgb_color: Option<[u8; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardState {
    pub entities: Vec<Entity>,
}
