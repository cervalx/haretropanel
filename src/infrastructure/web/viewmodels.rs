use std::collections::HashMap;

use askama::Template;
use serde::Serialize;

use crate::domain::{DashboardState, Entity, EntityKind};

#[derive(Debug, Serialize)]
pub struct EntityViewModel {
    pub id: String,
    pub name: String,
    pub kind_label: String,
    pub is_on: bool,
    pub has_value: bool,
    pub value: String,
    pub can_toggle: bool,
    pub brightness_pct: Option<u8>,
    pub supports_color_temp: bool,
    pub color_temp_kelvin: Option<u16>,
    pub min_color_temp_kelvin: Option<u16>,
    pub max_color_temp_kelvin: Option<u16>,
    pub supports_rgb: bool,
    pub rgb_hex: String,
    pub can_run_script: bool,
    pub action_label: String,
}

impl From<&Entity> for EntityViewModel {
    fn from(e: &Entity) -> Self {
        let kind_label = match e.kind {
            EntityKind::Light => "Light",
            EntityKind::Switch => "Switch",
            EntityKind::Sensor => "Sensor",
            EntityKind::Climate => "Climate",
            EntityKind::Script => "Script",
            EntityKind::Scene => "Scene",
        }
        .to_string();

        let (has_value, value) = if let Some(v) = &e.value {
            (true, v.clone())
        } else {
            (false, String::new())
        };

        let can_toggle = matches!(e.kind, EntityKind::Light | EntityKind::Switch);
        let brightness_pct = e.brightness.map(|value| ((value as u16 * 100 + 127) / 255) as u8);
        let supports_color_temp = e.color_temp_kelvin.is_some() || e.min_color_temp_kelvin.is_some() || e.max_color_temp_kelvin.is_some();
        let color_temp_kelvin = e.color_temp_kelvin;
        let min_color_temp_kelvin = e.min_color_temp_kelvin.or(Some(2000));
        let max_color_temp_kelvin = e.max_color_temp_kelvin.or(Some(6500));
        let supports_rgb = e.rgb_color.is_some();
        let rgb_hex = e
            .rgb_color
            .map(|[r, g, b]| format!("#{:02x}{:02x}{:02x}", r, g, b))
            .unwrap_or_else(|| "#ffffff".to_string());
        let can_run_script = matches!(e.kind, EntityKind::Script | EntityKind::Scene);
        let action_label = if matches!(e.kind, EntityKind::Scene) { "Activate".to_string() } else { "Run".to_string() };

        Self {
            id: e.id.to_string(),
            name: e.name.clone(),
            kind_label,
            is_on: e.is_on,
            has_value,
            value,
            can_toggle,
            brightness_pct,
            supports_color_temp,
            color_temp_kelvin,
            min_color_temp_kelvin,
            max_color_temp_kelvin,
            supports_rgb,
            rgb_hex,
            can_run_script,
            action_label,
        }
    }
}

#[derive(Serialize)]
pub struct DashboardPageViewModel {
    pub entities: Vec<EntityViewModel>,
    pub current_page: usize,
    pub total_pages: usize,
}

impl DashboardPageViewModel {
    pub fn from_state_and_pages(
        state: DashboardState,
        entity_pages: &HashMap<String, usize>,
        entity_order: &HashMap<String, usize>,
        requested_page: usize,
    ) -> Self {
        let mut items: Vec<(usize, usize, EntityViewModel)> = state
            .entities
            .iter()
            .map(|e| {
                let id = e.id.to_string();
                let page = entity_pages.get(&id).cloned().unwrap_or(1).max(1);
                let order = entity_order.get(&id).cloned().unwrap_or(usize::MAX);
                (page, order, EntityViewModel::from(e))
            })
            .collect();

        items.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.cmp(&b.1))
        });

        let total_pages = items.iter().map(|(p, _, _)| *p).max().unwrap_or(1);
        let page = requested_page.clamp(1, total_pages);

        let entities = items
            .into_iter()
            .filter(|(p, _, _)| *p == page)
            .map(|(_, _, vm)| vm)
            .collect();

        Self {
            entities,
            current_page: page,
            total_pages,
        }
    }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate<'a> {
    pub app_title: &'a str,
    pub app_text_color: &'a str,
    pub entities: &'a [EntityViewModel],
    pub current_page: usize,
    pub total_pages: usize,
}

#[derive(Debug)]
pub struct EntitySettingsViewModel {
    pub id: String,
    pub name: String,
    pub is_selected: bool,
    pub page: usize,
}

pub struct EntitiesSettingsPageViewModel {
    pub entities: Vec<EntitySettingsViewModel>,
}

#[derive(Template)]
#[template(path = "entities_settings.html")]
pub struct EntitiesSettingsTemplate<'a> {
    pub app_title: &'a str,
    pub app_text_color: &'a str,
    pub entities: &'a [EntitySettingsViewModel],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_view_model_exposes_color_controls() {
        let light = Entity {
            id: crate::domain::EntityId("light.lamp".to_string()),
            name: "Lamp".to_string(),
            kind: EntityKind::Light,
            is_on: true,
            value: None,
            brightness: Some(128),
            color_temp_kelvin: Some(3000),
            min_color_temp_kelvin: Some(2200),
            max_color_temp_kelvin: Some(6500),
            rgb_color: Some([255, 128, 64]),
        };

        let view = EntityViewModel::from(&light);

        assert_eq!(view.brightness_pct, Some(50));
        assert!(view.supports_color_temp);
        assert_eq!(view.color_temp_kelvin, Some(3000));
        assert!(view.supports_rgb);
        assert_eq!(view.rgb_hex, "#ff8040");
    }

    #[test]
    fn scene_view_model_uses_activation_button() {
        let scene = Entity {
            id: crate::domain::EntityId("scene.movie_night".to_string()),
            name: "Movie Night".to_string(),
            kind: EntityKind::Scene,
            is_on: false,
            value: None,
            brightness: None,
            color_temp_kelvin: None,
            min_color_temp_kelvin: None,
            max_color_temp_kelvin: None,
            rgb_color: None,
        };

        let view = EntityViewModel::from(&scene);

        assert_eq!(view.kind_label, "Scene");
        assert!(view.can_run_script);
        assert_eq!(view.action_label, "Activate");
    }
}
