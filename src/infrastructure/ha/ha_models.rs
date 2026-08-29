use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HaStateResponse {
    pub entity_id: String,
    pub state: String,
    
    #[serde(default)]
    pub attributes: HaAttributes,
}

#[derive(Debug, Deserialize, Default)]
pub struct HaAttributes {
    #[serde(default)]
    pub friendly_name: Option<String>,

    #[serde(default)]
    pub unit_of_measurement: Option<String>,

    #[serde(default)]
    pub brightness: Option<u8>,

    #[serde(default)]
    pub color_temp_kelvin: Option<u16>,

    #[serde(default)]
    pub min_color_temp_kelvin: Option<u16>,

    #[serde(default)]
    pub max_color_temp_kelvin: Option<u16>,

    #[serde(default)]
    pub rgb_color: Option<[u8; 3]>,
}
