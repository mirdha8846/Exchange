use redis::AsyncCommands;
use serde_json;
use shared::MatchEvent;
pub async fn publish_event(event: MatchEvent) {
    if let Ok(client) = redis::Client::open("redis://127.0.0.1/") {
        if let Ok(mut conn) = client.get_async_connection().await {
            let json = serde_json::to_string(&event).unwrap();
            let _: () = conn.lpush("event-queue", json).await.unwrap();
        }
    }}