mod socket_handler;

use std::sync::{Arc};
use axum::{
    Router,routing::get,
    // extract::{State,}
};
use socket_handler::{AppState, ws_handler, handle_event};
// use tokio::sync::broadcast;
//todo - events lena and unhe websocket and pub-sub donu ko send krna according to event type
use redis;
use shared::{MatchEvent};

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    // Redis setup
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let redis_conn = redis_client.get_async_connection().await.unwrap();
    let shared_conn = Arc::new(tokio::sync::Mutex::new(redis_conn));

    // Spawn Redis listener task
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            let mut conn = shared_conn.lock().await;
            let data: Option<(String, String)> = redis::cmd("BRPOP")
                .arg("event-queue")
                .arg(0)
                .query_async(&mut *conn)
                .await
                .ok();

            if let Some((_, raw)) = data {
                if let Ok(event) = serde_json::from_str::<MatchEvent>(&raw) {
                    handle_event(event, &state_clone).await;
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3500").await.unwrap();
    println!("Server running on http://0.0.0.0:3500");
    axum::serve(listener, app).await.unwrap();
}
