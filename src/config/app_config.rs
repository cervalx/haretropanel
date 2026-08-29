use crate::shared::error::{AppError, AppResult};
use dotenvy::dotenv;
use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub server_port: u16,
    pub ha_base_url: String,
    pub ha_token: Option<String>,
    pub app_title: String,
    pub app_text_color: String,

    // logging
    pub log_dir: String,
    pub log_rotation: LogRotation,
    pub log_level: String,

    // dashboard cache (in-memory, per entity kind)
    pub dashboard_cache_ttl_default_secs: u64,
    pub dashboard_cache_ttl_light_secs: Option<u64>,
    pub dashboard_cache_ttl_switch_secs: Option<u64>,
    pub dashboard_cache_ttl_sensor_secs: Option<u64>,
    pub dashboard_cache_ttl_climate_secs: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum LogRotation {
    Daily,
    Hourly,
    Never,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        dotenv().ok();

        let server_port = env::var("HARETROPANEL_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|e| AppError::Config(format!("Invalid HARETROPANEL_PORT: {e}")))?;

        let ha_base_url = env::var("HA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8123".to_string());

        let ha_token = env::var("HA_TOKEN").ok();

        let app_title = env::var("HARETROPANEL_TITLE")
            .unwrap_or_else(|_| "HARetroPanel - Home Assistant panel".to_string());

        let app_text_color = env::var("HARETROPANEL_TEXT_COLOR")
            .unwrap_or_else(|_| "#f0f0f0".to_string());

        let log_dir = env::var("HARETROPANEL_LOG_DIR")
            .unwrap_or_else(|_| "./logs".to_string());

        let log_rotation = match env::var("HARETROPANEL_LOG_ROTATION")
            .unwrap_or_else(|_| "daily".to_string())
            .to_lowercase()
            .as_str()
        {
            "hourly" => LogRotation::Hourly,
            "never" => LogRotation::Never,
            _ => LogRotation::Daily,
        };

        let log_level = env::var("HARETROPANEL_LOG_LEVEL")
            .unwrap_or_else(|_| "haretropanel=info,tower_http=info".to_string());

        // ---- Dashboard cache TTL config ----
        // Default TTL in seconds (used when no per-kind override exists)
        let dashboard_cache_ttl_default_secs = env::var("HARETROPANEL_CACHE_TTL_DEFAULT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|e| {
                AppError::Config(format!(
                    "Invalid HARETROPANEL_CACHE_TTL_DEFAULT_SECS: {e}"
                ))
            })?;

        // Helper closure to read optional u64 env vars.
        fn read_optional_u64(name: &str) -> AppResult<Option<u64>> {
            match env::var(name) {
                Ok(raw) => {
                    let value = raw.parse().map_err(|e| {
                        AppError::Config(format!("Invalid {name}: {e}"))
                    })?;
                    Ok(Some(value))
                }
                Err(_) => Ok(None),
            }
        }

        let dashboard_cache_ttl_light_secs =
            read_optional_u64("HARETROPANEL_CACHE_TTL_LIGHT_SECS")?;
        let dashboard_cache_ttl_switch_secs =
            read_optional_u64("HARETROPANEL_CACHE_TTL_SWITCH_SECS")?;
        let dashboard_cache_ttl_sensor_secs =
            read_optional_u64("HARETROPANEL_CACHE_TTL_SENSOR_SECS")?;
        let dashboard_cache_ttl_climate_secs =
            read_optional_u64("HARETROPANEL_CACHE_TTL_CLIMATE_SECS")?;

        Ok(Self {
            server_port,
            ha_base_url,
            ha_token,
            app_title,
            app_text_color,
            log_dir,
            log_rotation,
            log_level,
            dashboard_cache_ttl_default_secs,
            dashboard_cache_ttl_light_secs,
            dashboard_cache_ttl_switch_secs,
            dashboard_cache_ttl_sensor_secs,
            dashboard_cache_ttl_climate_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_title_respects_environment_override() {
        let previous = std::env::var("HARETROPANEL_TITLE").ok();
        std::env::set_var("HARETROPANEL_TITLE", "My Custom Title");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.app_title, "My Custom Title");

        match previous {
            Some(value) => std::env::set_var("HARETROPANEL_TITLE", value),
            None => std::env::remove_var("HARETROPANEL_TITLE"),
        }
    }

    #[test]
    fn text_color_respects_environment_override() {
        let previous = std::env::var("HARETROPANEL_TEXT_COLOR").ok();
        std::env::set_var("HARETROPANEL_TEXT_COLOR", "#00ff00");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.app_text_color, "#00ff00");

        match previous {
            Some(value) => std::env::set_var("HARETROPANEL_TEXT_COLOR", value),
            None => std::env::remove_var("HARETROPANEL_TEXT_COLOR"),
        }
    }
}
