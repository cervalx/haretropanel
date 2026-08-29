use axum::{
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;

use crate::infrastructure::web::{
        handlers::{
            dashboard_handler::{
                dashboard_websocket, get_dashboard, get_redirect_to_root, move_entity_down,
                move_entity_up, post_brightness, post_run_script, post_set_color,
                post_set_color_temp, post_toggle,
            },
            settings_handler::{get_entity_settings, post_entity_settings},
        },
    AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_dashboard))
        .route("/ws", get(dashboard_websocket))
        .route("/toggle",get(get_redirect_to_root).post(post_toggle),)
        .route("/brightness",get(get_redirect_to_root).post(post_brightness),)
        .route("/light/color_temp",get(get_redirect_to_root).post(post_set_color_temp),)
        .route("/light/color",get(get_redirect_to_root).post(post_set_color),)
        .route("/run_script",get(get_redirect_to_root).post(post_run_script),)
        .route("/entity/move_up",get(get_redirect_to_root).post(move_entity_up),)
        .route("/entity/move_down",get(get_redirect_to_root).post(move_entity_down),)
        .route("/settings/entities",get(get_entity_settings).post(post_entity_settings),)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
