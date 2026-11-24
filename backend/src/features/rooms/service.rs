use std::sync::Arc;

use chrono::{DateTime, Utc};
use entity::{room_commands, room_members, room_queue_items, rooms};
use migration::sea_orm::DatabaseConnection;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

use super::{
    domain::{
        self, ClientMessage, PLAYBACK_PAUSED, PLAYBACK_PLAYING, ROLE_MEMBER, ROLE_OWNER,
        RoomResponse, RoomState, RoomSummary, ServerMessage,
    },
    manager::RoomManager,
};

const MAX_QUEUE_ITEMS: usize = 100;

#[derive(Debug)]
pub struct CommandResult {
    pub response: RoomResponse,
    pub applied: bool,
}

pub struct RoomsService;

impl RoomsService {
    pub async fn create(db: &DatabaseConnection, user_id: &str) -> ApiResult<RoomResponse> {
        let room_id = Uuid::new_v4().to_string();
        let now = now();
        let room = rooms::ActiveModel {
            id: Set(room_id.clone()),
            owner_id: Set(user_id.to_owned()),
            join_code: Set(join_code()),
            state_version: Set(0),
            current_item_id: Set(None),
            playback_status: Set(PLAYBACK_PAUSED.to_owned()),
            playback_position_ms: Set(0),
            playback_changed_at: Set(now),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let member = room_members::ActiveModel {
            room_id: Set(room_id.clone()),
            user_id: Set(user_id.to_owned()),
            role: Set(ROLE_OWNER.to_owned()),
            joined_at: Set(now),
        };

        let transaction = db.begin().await.map_err(internal)?;
        room.insert(&transaction).await.map_err(internal)?;
        member.insert(&transaction).await.map_err(internal)?;
        transaction.commit().await.map_err(internal)?;

        Self::room_response(db, &room_id).await
    }

    pub async fn join(
        db: &DatabaseConnection,
        join_code: &str,
        user_id: &str,
    ) -> ApiResult<RoomResponse> {
        let room = rooms::Entity::find()
            .filter(rooms::Column::JoinCode.eq(join_code.trim().to_ascii_uppercase()))
            .one(db)
            .await
            .map_err(internal)?
            .ok_or_else(|| ApiError::NotFound("The room does not exist.".to_owned()))?;

        let member = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room.id.clone()))
            .filter(room_members::Column::UserId.eq(user_id.to_owned()))
            .one(db)
            .await
            .map_err(internal)?;

        if member.is_none() {
            room_members::ActiveModel {
                room_id: Set(room.id.clone()),
                user_id: Set(user_id.to_owned()),
                role: Set(ROLE_MEMBER.to_owned()),
                joined_at: Set(now()),
            }
            .insert(db)
            .await
            .map_err(internal)?;
        }

        Self::room_response(db, &room.id).await
    }

    pub async fn get(
        db: &DatabaseConnection,
        room_id: &str,
        user_id: &str,
    ) -> ApiResult<RoomResponse> {
        Self::require_member(db, room_id, user_id).await?;
        Self::room_response(db, room_id).await
    }

    pub async fn leave(db: &DatabaseConnection, room_id: &str, user_id: &str) -> ApiResult<()> {
        let role = Self::require_member(db, room_id, user_id).await?;
        if role == ROLE_OWNER {
            return Err(ApiError::Forbidden(
                "The owner must delete the room or transfer ownership first.".to_owned(),
            ));
        }

        room_members::Entity::delete_many()
            .filter(room_members::Column::RoomId.eq(room_id.to_owned()))
            .filter(room_members::Column::UserId.eq(user_id.to_owned()))
            .exec(db)
            .await
            .map_err(internal)?;
        Ok(())
    }

    pub async fn delete(
        db: &DatabaseConnection,
        manager: &Arc<Mutex<RoomManager>>,
        room_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let role = Self::require_member(db, room_id, user_id).await?;
        if role != ROLE_OWNER {
            return Err(ApiError::Forbidden(
                "Only the room owner can delete the room.".to_owned(),
            ));
        }

        let transaction = db.begin().await.map_err(internal)?;
        room_commands::Entity::delete_many()
            .filter(room_commands::Column::RoomId.eq(room_id.to_owned()))
            .exec(&transaction)
            .await
            .map_err(internal)?;
        room_queue_items::Entity::delete_many()
            .filter(room_queue_items::Column::RoomId.eq(room_id.to_owned()))
            .exec(&transaction)
            .await
            .map_err(internal)?;
        room_members::Entity::delete_many()
            .filter(room_members::Column::RoomId.eq(room_id.to_owned()))
            .exec(&transaction)
            .await
            .map_err(internal)?;
        rooms::Entity::delete_by_id(room_id.to_owned())
            .exec(&transaction)
            .await
            .map_err(internal)?;
        transaction.commit().await.map_err(internal)?;

        manager
            .lock()
            .await
            .publish(room_id, ServerMessage::RoomDeleted);
        Ok(())
    }

    pub async fn apply_command(
        db: &DatabaseConnection,
        manager: &Arc<Mutex<RoomManager>>,
        room_id: &str,
        user_id: &str,
        command: &ClientMessage,
    ) -> ApiResult<CommandResult> {
        let command_id = command.command_id().ok_or_else(|| {
            ApiError::BadRequest("This message is not a state-changing command.".to_owned())
        })?;
        if Uuid::parse_str(command_id).is_err() {
            return Err(ApiError::BadRequest("commandId must be a UUID.".to_owned()));
        }

        let _ = manager.lock().await.command_lock(room_id).lock().await;
        let command_hash = serde_json::to_string(command).map_err(internal)?;
        let transaction = db.begin().await.map_err(internal)?;

        let room = rooms::Entity::find_by_id(room_id.to_owned())
            .one(&transaction)
            .await
            .map_err(internal)?
            .ok_or_else(|| ApiError::NotFound("The room does not exist.".to_owned()))?;
        let member = room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id.to_owned()))
            .filter(room_members::Column::UserId.eq(user_id.to_owned()))
            .one(&transaction)
            .await
            .map_err(internal)?
            .ok_or_else(|| ApiError::Forbidden("You are not a member of this room.".to_owned()))?;

        if let Some(previous) = room_commands::Entity::find()
            .filter(room_commands::Column::RoomId.eq(room_id.to_owned()))
            .filter(room_commands::Column::CommandId.eq(command_id.to_owned()))
            .one(&transaction)
            .await
            .map_err(internal)?
        {
            if previous.command_hash != command_hash {
                return Err(ApiError::BadRequest(
                    "The command ID was already used for a different command.".to_owned(),
                ));
            }

            transaction.commit().await.map_err(internal)?;
            return Ok(CommandResult {
                response: Self::room_response(db, room_id).await?,
                applied: false,
            });
        }

        if let Some(expected_version) = command.expected_version()
            && expected_version != room.state_version
        {
            return Err(ApiError::Conflict(
                "The room changed before this command arrived.".to_owned(),
            ));
        }

        if command.command_type() != "queue.add" && member.role != ROLE_OWNER {
            match command {
                ClientMessage::QueueRemove { item_id, .. } => {
                    let item = find_item(&transaction, room_id, item_id).await?;
                    if item.added_by != user_id {
                        return Err(ApiError::Forbidden(
                            "You can remove only videos that you added.".to_owned(),
                        ));
                    }
                }
                _ => {
                    return Err(ApiError::Forbidden(
                        "Only the room owner can run this command.".to_owned(),
                    ));
                }
            }
        }

        let mut queue = load_queue(&transaction, room_id).await?;
        if matches!(command, ClientMessage::QueueAdd { .. }) && queue.len() >= MAX_QUEUE_ITEMS {
            return Err(ApiError::BadRequest(
                "This room has reached its queue limit.".to_owned(),
            ));
        }

        let now = now();
        let mut current_item_id = room.current_item_id.clone();
        let mut playback_status = room.playback_status.clone();
        let mut playback_position_ms = domain::effective_position(
            &room.playback_status,
            room.playback_position_ms,
            room.playback_changed_at,
            now,
        );
        let mut playback_changed_at = room.playback_changed_at;

        match command {
            ClientMessage::QueueAdd {
                video_id,
                before_item_id,
                ..
            } => {
                validate_video_id(video_id)?;
                let insert_at = insertion_index(&queue, before_item_id.as_deref())?;
                let item = room_queue_items::Model {
                    id: Uuid::new_v4().to_string(),
                    room_id: room_id.to_owned(),
                    video_id: video_id.clone(),
                    added_by: user_id.to_owned(),
                    sort_order: insert_at as i32,
                    created_at: now,
                };
                item.clone()
                    .into_active_model()
                    .insert(&transaction)
                    .await
                    .map_err(internal)?;
                queue.insert(insert_at, item);
            }
            ClientMessage::QueueRemove { item_id, .. } => {
                let index = queue_index(&queue, item_id)?;
                queue
                    .remove(index)
                    .into_active_model()
                    .delete(&transaction)
                    .await
                    .map_err(internal)?;
                if current_item_id.as_deref() == Some(item_id) {
                    current_item_id = queue.get(index).map(|item| item.id.clone());
                    playback_status = PLAYBACK_PAUSED.to_owned();
                    playback_position_ms = 0;
                    playback_changed_at = now;
                }
            }
            ClientMessage::QueueMove {
                item_id,
                before_item_id,
                ..
            } => {
                if before_item_id.as_deref() == Some(item_id) {
                    return Err(ApiError::BadRequest(
                        "An item cannot be moved before itself.".to_owned(),
                    ));
                }
                let index = queue_index(&queue, item_id)?;
                let item = queue.remove(index);
                let insert_at = insertion_index(&queue, before_item_id.as_deref())?;
                queue.insert(insert_at, item);
            }
            ClientMessage::PlaybackLoad {
                item_id,
                position_ms,
                autoplay,
                ..
            } => {
                queue_index(&queue, item_id)?;
                let position = position_ms.unwrap_or(0);
                if position < 0 {
                    return Err(ApiError::BadRequest(
                        "positionMs must be non-negative.".to_owned(),
                    ));
                }
                current_item_id = Some(item_id.clone());
                playback_position_ms = position;
                playback_status = if autoplay.unwrap_or(false) {
                    PLAYBACK_PLAYING.to_owned()
                } else {
                    PLAYBACK_PAUSED.to_owned()
                };
                playback_changed_at = now;
            }
            ClientMessage::PlaybackPlay { .. } => {
                if current_item_id.is_none() {
                    return Err(ApiError::BadRequest(
                        "Load a queue item before starting playback.".to_owned(),
                    ));
                }
                playback_status = PLAYBACK_PLAYING.to_owned();
                playback_changed_at = now;
            }
            ClientMessage::PlaybackPause { .. } => {
                playback_status = PLAYBACK_PAUSED.to_owned();
                playback_changed_at = now;
            }
            ClientMessage::PlaybackSeek { position_ms, .. } => {
                if *position_ms < 0 {
                    return Err(ApiError::BadRequest(
                        "positionMs must be non-negative.".to_owned(),
                    ));
                }
                playback_position_ms = *position_ms;
                playback_changed_at = now;
            }
            ClientMessage::PlaybackNext { .. } => {
                let next = next_item(&queue, current_item_id.as_deref());
                current_item_id = next.map(|item| item.id.clone());
                playback_position_ms = 0;
                playback_changed_at = now;
                if current_item_id.is_none() {
                    playback_status = PLAYBACK_PAUSED.to_owned();
                }
            }
            ClientMessage::PlaybackEnded { item_id, .. } => {
                if current_item_id.as_deref() != Some(item_id) {
                    return Err(ApiError::BadRequest(
                        "The ended item is not the current room item.".to_owned(),
                    ));
                }
                let next = next_item(&queue, current_item_id.as_deref());
                current_item_id = next.map(|item| item.id.clone());
                playback_position_ms = 0;
                playback_changed_at = now;
                playback_status = if current_item_id.is_some() {
                    PLAYBACK_PLAYING.to_owned()
                } else {
                    PLAYBACK_PAUSED.to_owned()
                };
            }
            ClientMessage::StateRequest | ClientMessage::ClientPong => {
                return Err(ApiError::BadRequest(
                    "This message is not a command.".to_owned(),
                ));
            }
        }

        renumber_queue(&transaction, &queue).await?;
        let next_version = room.state_version + 1;
        let mut active_room = room.into_active_model();
        active_room.current_item_id = Set(current_item_id);
        active_room.playback_status = Set(playback_status);
        active_room.playback_position_ms = Set(playback_position_ms);
        active_room.playback_changed_at = Set(playback_changed_at);
        active_room.state_version = Set(next_version);
        active_room.updated_at = Set(now);
        active_room.update(&transaction).await.map_err(internal)?;

        room_commands::ActiveModel {
            room_id: Set(room_id.to_owned()),
            command_id: Set(command_id.to_owned()),
            user_id: Set(user_id.to_owned()),
            command_type: Set(command.command_type().to_owned()),
            command_hash: Set(command_hash),
            result_version: Set(next_version),
            created_at: Set(now),
        }
        .insert(&transaction)
        .await
        .map_err(internal)?;

        transaction.commit().await.map_err(internal)?;
        Ok(CommandResult {
            response: Self::room_response(db, room_id).await?,
            applied: true,
        })
    }

    pub async fn require_member(
        db: &DatabaseConnection,
        room_id: &str,
        user_id: &str,
    ) -> ApiResult<String> {
        room_members::Entity::find()
            .filter(room_members::Column::RoomId.eq(room_id.to_owned()))
            .filter(room_members::Column::UserId.eq(user_id.to_owned()))
            .one(db)
            .await
            .map_err(internal)?
            .map(|member| member.role)
            .ok_or_else(|| ApiError::Forbidden("You are not a member of this room.".to_owned()))
    }

    pub async fn room_response(db: &DatabaseConnection, room_id: &str) -> ApiResult<RoomResponse> {
        let room = rooms::Entity::find_by_id(room_id.to_owned())
            .one(db)
            .await
            .map_err(internal)?
            .ok_or_else(|| ApiError::NotFound("The room does not exist.".to_owned()))?;
        let queue = load_queue(db, room_id).await?;
        let now = now();
        let position_ms = domain::effective_position(
            &room.playback_status,
            room.playback_position_ms,
            room.playback_changed_at,
            now,
        );

        Ok(RoomResponse {
            room: RoomSummary {
                id: room.id,
                join_code: room.join_code,
                owner_id: room.owner_id,
                created_at: room.created_at.to_rfc3339(),
            },
            state: RoomState {
                state_version: room.state_version,
                server_time: now.to_rfc3339(),
                queue: queue
                    .into_iter()
                    .map(|item| domain::QueueItemView {
                        id: item.id,
                        video_id: item.video_id,
                        added_by: item.added_by,
                        sort_order: item.sort_order,
                        created_at: item.created_at.to_rfc3339(),
                    })
                    .collect(),
                playback: domain::PlaybackView {
                    item_id: room.current_item_id,
                    status: room.playback_status,
                    position_ms,
                    changed_at: room.playback_changed_at.to_rfc3339(),
                },
            },
        })
    }
}

fn internal(error: impl Into<anyhow::Error>) -> ApiError {
    ApiError::InternalError(error.into())
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

fn join_code() -> String {
    Uuid::new_v4().simple().to_string()[..8].to_ascii_uppercase()
}

async fn load_queue<C: ConnectionTrait>(
    connection: &C,
    room_id: &str,
) -> ApiResult<Vec<room_queue_items::Model>> {
    room_queue_items::Entity::find()
        .filter(room_queue_items::Column::RoomId.eq(room_id.to_owned()))
        .order_by_asc(room_queue_items::Column::SortOrder)
        .all(connection)
        .await
        .map_err(internal)
}

async fn find_item<C: ConnectionTrait>(
    connection: &C,
    room_id: &str,
    item_id: &str,
) -> ApiResult<room_queue_items::Model> {
    room_queue_items::Entity::find_by_id(item_id.to_owned())
        .filter(room_queue_items::Column::RoomId.eq(room_id.to_owned()))
        .one(connection)
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::NotFound("The queue item does not exist.".to_owned()))
}

async fn renumber_queue<C: ConnectionTrait>(
    connection: &C,
    queue: &[room_queue_items::Model],
) -> ApiResult<()> {
    for (index, item) in queue.iter().enumerate() {
        let mut active = item.clone().into_active_model();
        active.sort_order = Set(index as i32);
        active.update(connection).await.map_err(internal)?;
    }
    Ok(())
}

fn validate_video_id(video_id: &str) -> ApiResult<()> {
    let valid = video_id.len() == 11
        && video_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "videoId must be an 11-character YouTube video ID.".to_owned(),
        ))
    }
}

fn queue_index(queue: &[room_queue_items::Model], item_id: &str) -> ApiResult<usize> {
    queue
        .iter()
        .position(|item| item.id == item_id)
        .ok_or_else(|| ApiError::NotFound("The queue item does not exist.".to_owned()))
}

fn insertion_index(
    queue: &[room_queue_items::Model],
    before_item_id: Option<&str>,
) -> ApiResult<usize> {
    before_item_id
        .map(|item_id| queue_index(queue, item_id))
        .transpose()
        .map(|index| index.unwrap_or(queue.len()))
}

fn next_item<'a>(
    queue: &'a [room_queue_items::Model],
    current_item_id: Option<&str>,
) -> Option<&'a room_queue_items::Model> {
    match current_item_id {
        Some(current_id) => queue
            .iter()
            .position(|item| item.id == current_id)
            .and_then(|index| queue.get(index + 1)),
        None => queue.first(),
    }
}
