use std::sync::{Arc};

use redis::AsyncCommands;
use serde_json;
use shared::MatchEvent;
pub async fn publish_event(event: MatchEvent, conn: &Arc<tokio::sync::Mutex<redis::aio::Connection>>) {
    let json = serde_json::to_string(&event).unwrap();
    let mut conn = conn.lock().await;

    let _: () = conn.lpush("event-queue", json).await.unwrap();
}