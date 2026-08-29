use askama::Template;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use tokio::time::{interval, Duration};
use tracing::info;

use crate::{
    domain::EntityId,
    infrastructure::web::{
        viewmodels::{DashboardPageViewModel, DashboardTemplate},
        AppState,
    },
    shared::error::AppResult,
};

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub page: Option<usize>,
    pub force_refresh: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleForm {
    pub entity_id: String,
    #[serde(default)]
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct BrightnessForm {
    pub entity_id: String,
    pub brightness_pct: u8,
    #[serde(default)]
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RunScriptForm {
    pub entity_id: String,
    #[serde(default)]
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct MoveEntityForm {
    pub entity_id: String,
    #[serde(default)]
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ColorTempForm {
    pub entity_id: String,
    pub color_temp_kelvin: u16,
    #[serde(default)]
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ColorForm {
    pub entity_id: String,
    pub rgb: String,
    #[serde(default)]
    pub page: Option<usize>,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> AppResult<impl IntoResponse> {
    let requested_page = query.page.unwrap_or(1);
    let force_refresh = query.force_refresh.is_some();

    let dashboard_state = state
        .dashboard_service
        .get_dashboard_with_refresh(force_refresh)
        .await?;
    let entity_pages = state.dashboard_service.get_entity_pages().await?;
    let entity_order = state.dashboard_service.get_entity_order().await?;

    let vm = DashboardPageViewModel::from_state_and_pages(
        dashboard_state,
        &entity_pages,
        &entity_order,
        requested_page,
    );

    let template = DashboardTemplate {
        app_title: &state.app_title,
        app_text_color: &state.app_text_color,
        entities: &vm.entities,
        current_page: vm.current_page,
        total_pages: vm.total_pages,
    };

    let rendered = template.render()?;
    Ok(Html(rendered))
}

pub async fn dashboard_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> impl IntoResponse {
    let requested_page = query.page.unwrap_or(1);
    ws.on_upgrade(move |socket| serve_dashboard_websocket(socket, state, requested_page))
}

async fn serve_dashboard_websocket(
    mut socket: WebSocket,
    state: AppState,
    requested_page: usize,
) {
    let mut refresh_interval = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = refresh_interval.tick() => {
                let snapshot = async {
                    let dashboard_state = state.dashboard_service.get_dashboard().await?;
                    let entity_pages = state.dashboard_service.get_entity_pages().await?;
                    let entity_order = state.dashboard_service.get_entity_order().await?;
                    let vm = DashboardPageViewModel::from_state_and_pages(
                        dashboard_state,
                        &entity_pages,
                        &entity_order,
                        requested_page,
                    );
                    serde_json::to_string(&vm)
                        .map_err(|error| crate::shared::error::AppError::Internal(error.to_string()))
                }.await;

                match snapshot {
                    Ok(json) => {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(error) => tracing::warn!("Dashboard WebSocket refresh failed: {error}"),
                }
            }
            message = socket.recv() => {
                match message {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

pub async fn post_set_color_temp(
    State(state): State<AppState>,
    Form(form): Form<ColorTempForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id.clone());
    info!(
        "Setting color temperature via POST /light/color_temp: {} to {}K",
        id.0,
        form.color_temp_kelvin
    );
    state
        .dashboard_service
        .set_color_temp(&id, form.color_temp_kelvin)
        .await?;
    let redirect_target = redirect_target_for_page(form.page);
    Ok(Redirect::to(&redirect_target))
}

pub async fn post_set_color(
    State(state): State<AppState>,
    Form(form): Form<ColorForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id.clone());
    let rgb = parse_hex_color(&form.rgb)?;
    info!("Setting color via POST /light/color: {} to #{:02x}{:02x}{:02x}", id.0, rgb[0], rgb[1], rgb[2]);
    state.dashboard_service.set_color(&id, rgb).await?;
    let redirect_target = redirect_target_for_page(form.page);
    Ok(Redirect::to(&redirect_target))
}

fn parse_hex_color(raw: &str) -> AppResult<[u8; 3]> {
    let value = raw.trim();
    let without_hash = value.strip_prefix('#').unwrap_or(value);
    if without_hash.len() != 6 {
        return Err(crate::shared::error::AppError::Internal(format!(
            "Invalid color value: {raw}"
        )));
    }

    let r = u8::from_str_radix(&without_hash[0..2], 16)
        .map_err(|e| crate::shared::error::AppError::Internal(format!("Invalid color value {raw}: {e}")))?;
    let g = u8::from_str_radix(&without_hash[2..4], 16)
        .map_err(|e| crate::shared::error::AppError::Internal(format!("Invalid color value {raw}: {e}")))?;
    let b = u8::from_str_radix(&without_hash[4..6], 16)
        .map_err(|e| crate::shared::error::AppError::Internal(format!("Invalid color value {raw}: {e}")))?;

    Ok([r, g, b])
}

fn redirect_target_for_page(page: Option<usize>) -> String {
    format!("/?page={}&force_refresh=1", page.unwrap_or(1))
}

pub async fn post_toggle(
    State(state): State<AppState>,
    Form(form): Form<ToggleForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id);
    info!("Toggling entity via POST /toggle: {}", id);

    state.dashboard_service.toggle_entity(&id).await?;

    let redirect_target = redirect_target_for_page(form.page);
    Ok(Redirect::to(&redirect_target))
}

pub async fn post_brightness(
    State(state): State<AppState>,
    Form(form): Form<BrightnessForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id);
    info!("Setting brightness via POST /brightness: {} to {}%", id, form.brightness_pct);

    state
        .dashboard_service
        .set_brightness(&id, form.brightness_pct)
        .await?;

    let redirect_target = redirect_target_for_page(form.page);
    Ok(Redirect::to(&redirect_target))
}

pub async fn post_run_script(
    State(state): State<AppState>,
    Form(form): Form<RunScriptForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id);
    info!("Running script via POST /run_script: {}", id);

    state.dashboard_service.run_script(&id).await?;

    let redirect_target = redirect_target_for_page(form.page);
    Ok(Redirect::to(&redirect_target))
}

pub async fn move_entity_up(
    State(state): State<AppState>,
    Form(form): Form<MoveEntityForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id.clone());
    let entity_pages = state.dashboard_service.get_entity_pages().await?;
    let page = entity_pages.get(&id.0).cloned().unwrap_or(1);

    let dashboard_state = state.dashboard_service.get_dashboard().await?;
    let mut page_ids: Vec<String> = dashboard_state
        .entities
        .into_iter()
        .filter(|e| entity_pages.get(&e.id.0).cloned().unwrap_or(1) == page)
        .map(|e| e.id.0)
        .collect();

    let current_index = page_ids
        .iter()
        .position(|entity_id| entity_id == &id.0)
        .unwrap_or(0);
    if current_index > 0 {
        page_ids.swap(current_index, current_index - 1);
    }

    let entity_order = state.dashboard_service.get_entity_order().await?;
    let mut new_order = entity_order.clone();
    for (index, entity_id) in page_ids.iter().enumerate() {
        new_order.insert(entity_id.clone(), index + 1);
    }
    state.dashboard_service.save_entity_order(new_order).await?;

    let redirect_target = redirect_target_for_page(Some(page));
    Ok(Redirect::to(&redirect_target))
}

pub async fn move_entity_down(
    State(state): State<AppState>,
    Form(form): Form<MoveEntityForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id.clone());
    let entity_pages = state.dashboard_service.get_entity_pages().await?;
    let page = entity_pages.get(&id.0).cloned().unwrap_or(1);

    let dashboard_state = state.dashboard_service.get_dashboard().await?;
    let mut page_ids: Vec<String> = dashboard_state
        .entities
        .into_iter()
        .filter(|e| entity_pages.get(&e.id.0).cloned().unwrap_or(1) == page)
        .map(|e| e.id.0)
        .collect();

    let current_index = page_ids
        .iter()
        .position(|entity_id| entity_id == &id.0)
        .unwrap_or(0);
    if current_index + 1 < page_ids.len() {
        page_ids.swap(current_index, current_index + 1);
    }

    let entity_order = state.dashboard_service.get_entity_order().await?;
    let mut new_order = entity_order.clone();
    for (index, entity_id) in page_ids.iter().enumerate() {
        new_order.insert(entity_id.clone(), index + 1);
    }
    state.dashboard_service.save_entity_order(new_order).await?;

    let redirect_target = redirect_target_for_page(Some(page));
    Ok(Redirect::to(&redirect_target))
}

pub async fn get_redirect_to_root() -> impl IntoResponse {
    Redirect::to("/")
}