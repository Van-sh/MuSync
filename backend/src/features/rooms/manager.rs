use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};

use super::domain::ServerMessage;

const CHANNEL_CAPACITY: usize = 128;

struct RoomRuntime {
    updates: broadcast::Sender<ServerMessage>,
    command_lock: Arc<Mutex<()>>,
}

#[derive(Default)]
pub struct RoomManager {
    rooms: HashMap<String, RoomRuntime>,
}

impl RoomManager {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn subscribe(&mut self, room_id: &str) -> broadcast::Receiver<ServerMessage> {
        self.runtime(room_id).updates.subscribe()
    }

    pub fn publish(&mut self, room_id: &str, message: ServerMessage) {
        let _ = self.runtime(room_id).updates.send(message);
    }

    pub fn command_lock(&mut self, room_id: &str) -> Arc<Mutex<()>> {
        self.runtime(room_id).command_lock.clone()
    }

    fn runtime(&mut self, room_id: &str) -> &mut RoomRuntime {
        self.rooms.entry(room_id.to_owned()).or_insert_with(|| {
            let (updates, _) = broadcast::channel(CHANNEL_CAPACITY);
            RoomRuntime {
                updates,
                command_lock: Arc::new(Mutex::new(())),
            }
        })
    }
}
