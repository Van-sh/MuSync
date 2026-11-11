use axum::Router;

use crate::app::AppState;

pub struct RoomsRouter;

impl RoomsRouter {
    pub fn get() -> Router<AppState> {
        // TODO: get all with filter
        // TODO: create room
        // TODO: websocket (join, sync, leave)
        // TODO: delete room
        Router::new()
    }
}
