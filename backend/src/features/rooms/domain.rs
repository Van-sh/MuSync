use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const ROLE_OWNER: &str = "owner";
pub const ROLE_MEMBER: &str = "member";
pub const PLAYBACK_PAUSED: &str = "paused";
pub const PLAYBACK_PLAYING: &str = "playing";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    #[serde(rename = "state.request")]
    StateRequest,
    #[serde(rename = "client.pong")]
    ClientPong,
    #[serde(rename = "queue.add")]
    QueueAdd {
        command_id: String,
        expected_version: Option<i64>,
        video_id: String,
        before_item_id: Option<String>,
    },
    #[serde(rename = "queue.remove")]
    QueueRemove {
        command_id: String,
        expected_version: Option<i64>,
        item_id: String,
    },
    #[serde(rename = "queue.move")]
    QueueMove {
        command_id: String,
        expected_version: Option<i64>,
        item_id: String,
        before_item_id: Option<String>,
    },
    #[serde(rename = "playback.load")]
    PlaybackLoad {
        command_id: String,
        expected_version: Option<i64>,
        item_id: String,
        position_ms: Option<i64>,
        autoplay: Option<bool>,
    },
    #[serde(rename = "playback.play")]
    PlaybackPlay {
        command_id: String,
        expected_version: Option<i64>,
    },
    #[serde(rename = "playback.pause")]
    PlaybackPause {
        command_id: String,
        expected_version: Option<i64>,
    },
    #[serde(rename = "playback.seek")]
    PlaybackSeek {
        command_id: String,
        expected_version: Option<i64>,
        position_ms: i64,
    },
    #[serde(rename = "playback.next")]
    PlaybackNext {
        command_id: String,
        expected_version: Option<i64>,
    },
    #[serde(rename = "playback.ended")]
    PlaybackEnded {
        command_id: String,
        expected_version: Option<i64>,
        item_id: String,
    },
}

impl ClientMessage {
    pub fn command_id(&self) -> Option<&str> {
        match self {
            Self::StateRequest | Self::ClientPong => None,
            Self::QueueAdd { command_id, .. }
            | Self::QueueRemove { command_id, .. }
            | Self::QueueMove { command_id, .. }
            | Self::PlaybackLoad { command_id, .. }
            | Self::PlaybackPlay { command_id, .. }
            | Self::PlaybackPause { command_id, .. }
            | Self::PlaybackSeek { command_id, .. }
            | Self::PlaybackNext { command_id, .. }
            | Self::PlaybackEnded { command_id, .. } => Some(command_id),
        }
    }

    pub fn command_type(&self) -> &'static str {
        match self {
            Self::StateRequest => "state.request",
            Self::ClientPong => "client.pong",
            Self::QueueAdd { .. } => "queue.add",
            Self::QueueRemove { .. } => "queue.remove",
            Self::QueueMove { .. } => "queue.move",
            Self::PlaybackLoad { .. } => "playback.load",
            Self::PlaybackPlay { .. } => "playback.play",
            Self::PlaybackPause { .. } => "playback.pause",
            Self::PlaybackSeek { .. } => "playback.seek",
            Self::PlaybackNext { .. } => "playback.next",
            Self::PlaybackEnded { .. } => "playback.ended",
        }
    }

    pub fn expected_version(&self) -> Option<i64> {
        match self {
            Self::StateRequest | Self::ClientPong => None,
            Self::QueueAdd {
                expected_version, ..
            }
            | Self::QueueRemove {
                expected_version, ..
            }
            | Self::QueueMove {
                expected_version, ..
            }
            | Self::PlaybackLoad {
                expected_version, ..
            }
            | Self::PlaybackPlay {
                expected_version, ..
            }
            | Self::PlaybackPause {
                expected_version, ..
            }
            | Self::PlaybackSeek {
                expected_version, ..
            }
            | Self::PlaybackNext {
                expected_version, ..
            }
            | Self::PlaybackEnded {
                expected_version, ..
            } => *expected_version,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    #[serde(rename = "state.sync")]
    StateSync {
        reason: &'static str,
        command_id: Option<String>,
        state: RoomState,
    },
    #[serde(rename = "server.error")]
    Error {
        command_id: Option<String>,
        code: &'static str,
        message: String,
        state: Option<RoomState>,
    },
    #[serde(rename = "room.deleted")]
    RoomDeleted,
    #[serde(rename = "server.ping")]
    Ping,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomState {
    pub state_version: i64,
    pub server_time: String,
    pub queue: Vec<QueueItemView>,
    pub playback: PlaybackView,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSummary {
    pub id: String,
    pub join_code: String,
    pub owner_id: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemView {
    pub id: String,
    pub video_id: String,
    pub added_by: String,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackView {
    pub item_id: Option<String>,
    pub status: String,
    pub position_ms: i64,
    pub changed_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RoomResponse {
    pub room: RoomSummary,
    pub state: RoomState,
}

pub fn effective_position(
    status: &str,
    position_ms: i64,
    changed_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> i64 {
    if status != PLAYBACK_PLAYING {
        return position_ms.max(0);
    }

    position_ms
        .saturating_add(
            now.signed_duration_since(changed_at)
                .num_milliseconds()
                .max(0),
        )
        .max(0)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{PLAYBACK_PAUSED, PLAYBACK_PLAYING, effective_position};

    #[test]
    fn paused_position_does_not_advance() {
        let changed_at = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 1, 0).unwrap();

        assert_eq!(
            effective_position(PLAYBACK_PAUSED, 2_000, changed_at, now),
            2_000
        );
    }

    #[test]
    fn playing_position_advances_from_the_change_time() {
        let changed_at = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 12).unwrap();

        assert_eq!(
            effective_position(PLAYBACK_PLAYING, 2_000, changed_at, now),
            14_000
        );
    }
}
