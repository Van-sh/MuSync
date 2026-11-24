use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use better_auth::integrations::axum::CurrentSession;
use futures_util::{SinkExt, StreamExt};
use migration::sea_orm::DatabaseConnection;
use tokio::{
    sync::{Mutex, broadcast::error::RecvError, mpsc},
    time::interval,
};

use crate::{
    app::AppState,
    error::{ApiError, ApiResult},
    features::{
        auth::AppAuthSchema,
        rooms::{
            domain::{ClientMessage, RoomState, ServerMessage},
            manager::RoomManager,
            service::RoomsService,
        },
    },
};

pub async fn connect(
    State(state): State<AppState>,
    session: CurrentSession<AppAuthSchema>,
    Path(room_id): Path<String>,
    websocket: WebSocketUpgrade,
) -> ApiResult<Response> {
    RoomsService::require_member(&state.db, &room_id, &session.user.id).await?;

    let db = state.db.clone();
    let manager = state.room_manager.clone();
    let user_id = session.user.id.clone();
    Ok(websocket.on_upgrade(move |socket| run(socket, db, manager, room_id, user_id)))
}

async fn run(
    socket: WebSocket,
    db: DatabaseConnection,
    manager: Arc<Mutex<RoomManager>>,
    room_id: String,
    user_id: String,
) {
    let mut updates = manager.lock().await.subscribe(&room_id);
    let (mut socket_sender, mut socket_receiver) = socket.split();
    let (outgoing, mut outgoing_receiver) = mpsc::channel::<Message>(64);

    let writer = tokio::spawn(async move {
        while let Some(message) = outgoing_receiver.recv().await {
            if socket_sender.send(message).await.is_err() {
                break;
            }
        }
    });

    match RoomsService::room_response(&db, &room_id).await {
        Ok(response) => {
            let _ = send(
                &outgoing,
                ServerMessage::StateSync {
                    reason: "initial",
                    command_id: None,
                    state: response.state,
                },
            )
            .await;
        }
        Err(error) => {
            let _ = send(&outgoing, server_error(None, &error, None)).await;
            writer.abort();
            return;
        }
    }

    let mut heartbeat = interval(Duration::from_secs(30));
    loop {
        tokio::select! {
            message = socket_receiver.next() => {
                let Some(Ok(message)) = message else { break };
                match message {
                    Message::Text(text) => handle_client_message(&outgoing, &db, &manager, &room_id, &user_id, &text).await,
                    Message::Close(_) => break,
                    Message::Ping(payload) => {
                        if outgoing.send(Message::Pong(payload)).await.is_err() { break; }
                    }
                    Message::Binary(_) | Message::Pong(_) => {}
                }
            }
            update = updates.recv() => {
                match update {
                    Ok(message) => {
                        if send(&outgoing, message).await.is_err() { break; }
                    }
                    Err(RecvError::Lagged(_)) => {
                        if let Ok(response) = RoomsService::room_response(&db, &room_id).await &&
                             send(&outgoing, ServerMessage::StateSync { reason: "resync", command_id: None, state: response.state }).await.is_err() { break;
                        }
                    }
                    Err(RecvError::Closed) => break,
                }
            }
            _ = heartbeat.tick() => {
                if send(&outgoing, ServerMessage::Ping).await.is_err() { break; }
            }
        }
    }

    writer.abort();
}

async fn handle_client_message(
    outgoing: &mpsc::Sender<Message>,
    db: &DatabaseConnection,
    manager: &Arc<Mutex<RoomManager>>,
    room_id: &str,
    user_id: &str,
    text: &str,
) {
    let message = match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => message,
        Err(error) => {
            let _ = send(
                outgoing,
                ServerMessage::Error {
                    command_id: None,
                    code: "BAD_REQUEST",
                    message: format!("Invalid message: {error}"),
                    state: None,
                },
            )
            .await;
            return;
        }
    };

    if matches!(message, ClientMessage::ClientPong) {
        return;
    }

    if matches!(message, ClientMessage::StateRequest) {
        match RoomsService::room_response(db, room_id).await {
            Ok(response) => {
                let _ = send(
                    outgoing,
                    ServerMessage::StateSync {
                        reason: "requested",
                        command_id: None,
                        state: response.state,
                    },
                )
                .await;
            }
            Err(error) => {
                let _ = send(outgoing, server_error(None, &error, None)).await;
            }
        }
        return;
    }

    let command_id = message.command_id().map(str::to_owned);
    match RoomsService::apply_command(db, manager, room_id, user_id, &message).await {
        Ok(result) if result.applied => {
            manager.lock().await.publish(
                room_id,
                ServerMessage::StateSync {
                    reason: "command.accepted",
                    command_id,
                    state: result.response.state,
                },
            );
        }
        Ok(result) => {
            let _ = send(
                outgoing,
                ServerMessage::StateSync {
                    reason: "command.duplicate",
                    command_id,
                    state: result.response.state,
                },
            )
            .await;
        }
        Err(error) => {
            let state = if matches!(error, ApiError::Conflict(_)) {
                RoomsService::room_response(db, room_id)
                    .await
                    .ok()
                    .map(|response| response.state)
            } else {
                None
            };
            let _ = send(outgoing, server_error(command_id, &error, state)).await;
        }
    }
}

fn server_error(
    command_id: Option<String>,
    error: &ApiError,
    state: Option<RoomState>,
) -> ServerMessage {
    let (code, message) = match error {
        ApiError::BadRequest(message) => ("BAD_REQUEST", message.clone()),
        ApiError::Forbidden(message) => ("FORBIDDEN", message.clone()),
        ApiError::NotFound(message) => ("NOT_FOUND", message.clone()),
        ApiError::Conflict(message) => ("CONFLICT", message.clone()),
        ApiError::InternalError(error) => {
            ("INTERNAL_ERROR", format!("Something went wrong: {error}"))
        }
    };
    ServerMessage::Error {
        command_id,
        code,
        message,
        state,
    }
}

async fn send(outgoing: &mpsc::Sender<Message>, message: ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(&message).map_err(|_| ())?;
    outgoing
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}
