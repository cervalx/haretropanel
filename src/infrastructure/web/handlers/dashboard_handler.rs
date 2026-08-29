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
}

#[derive(Debug, Deserialize)]
pub struct BrightnessForm {
    pub entity_id: String,
    pub brightness_pct: u8,
}

#[derive(Debug, Deserialize)]
pub struct RunScriptForm {
    pub entity_id: String,
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

    let vm =
        DashboardPageViewModel::from_state_and_pages(dashboard_state, &entity_pages, requested_page);

    let template = DashboardTemplate {
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
                    let vm = DashboardPageViewModel::from_state_and_pages(
                        dashboard_state,
                        &entity_pages,
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

pub async fn post_toggle(
    State(state): State<AppState>,
    Form(form): Form<ToggleForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id);
    info!("Toggling entity via POST /toggle: {}", id);

    state.dashboard_service.toggle_entity(&id).await?;

    Ok(Redirect::to("/"))
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

    Ok(Redirect::to("/"))
}

pub async fn post_run_script(
    State(state): State<AppState>,
    Form(form): Form<RunScriptForm>,
) -> AppResult<impl IntoResponse> {
    let id = EntityId(form.entity_id);
    info!("Running script via POST /run_script: {}", id);

    state.dashboard_service.run_script(&id).await?;

    Ok(Redirect::to("/"))
}

pub async fn get_redirect_to_root() -> impl IntoResponse {
    Redirect::to("/")
}