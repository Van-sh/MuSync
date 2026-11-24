use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use better_auth::integrations::axum::CurrentSession;
use serde::Deserialize;

use crate::{
    app::AppState,
    error::ApiResult,
    features::{
        auth::AppAuthSchema,
        rooms::{domain::RoomResponse, service::RoomsService},
    },
};

use super::websocket;

pub struct RoomsRouter;

impl RoomsRouter {
    pub fn get() -> Router<AppState> {
        Router::new()
            .route("/", post(create_room))
            .route("/join", post(join_room_by_id))
            .route(
                "/{room_id}",
                get(get_room_data_by_id).delete(delete_room_by_id),
            )
            .route("/{room_id}/membership", delete(leave_room_by_id))
            .route("/{room_id}/ws", get(websocket::connect))
    }
}

async fn create_room(
    State(state): State<AppState>,
    session: CurrentSession<AppAuthSchema>,
) -> ApiResult<Json<RoomResponse>> {
    Ok(Json(
        RoomsService::create(&state.db, &session.user.id).await?,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JoinRoomRequest {
    pub join_code: String,
}

async fn join_room_by_id(
    State(state): State<AppState>,
    session: CurrentSession<AppAuthSchema>,
    Json(request): Json<JoinRoomRequest>,
) -> ApiResult<Json<RoomResponse>> {
    Ok(Json(
        RoomsService::join(&state.db, &request.join_code, &session.user.id).await?,
    ))
}

async fn get_room_data_by_id(
    State(state): State<AppState>,
    session: CurrentSession<AppAuthSchema>,
    Path(room_id): Path<String>,
) -> ApiResult<Json<RoomResponse>> {
    Ok(Json(
        RoomsService::get(&state.db, &room_id, &session.user.id).await?,
    ))
}

pub async fn delete_room_by_id(
    State(state): State<AppState>,
    session: CurrentSession<AppAuthSchema>,
    Path(room_id): Path<String>,
) -> ApiResult<StatusCode> {
    RoomsService::delete(&state.db, &state.room_manager, &room_id, &session.user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn leave_room_by_id(
    State(state): State<AppState>,
    session: CurrentSession<AppAuthSchema>,
    Path(room_id): Path<String>,
) -> ApiResult<StatusCode> {
    RoomsService::leave(&state.db, &room_id, &session.user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
