use std::sync::Arc;

use tokio::sync::Mutex;

pub struct RoomManager;

impl RoomManager {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(RoomManager))
    }
}
