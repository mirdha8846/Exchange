mod socket_handler;
use sysinfo::{System};
use std::sync::{Arc};
use axum::{
    response::IntoResponse, routing::get, Router
    // extract::{State,}
};
use socket_handler::{AppState, ws_handler, handle_event};
// use tokio::sync::broadcast;
//todo - events lena and unhe websocket and pub-sub donu ko send krna according to event type
use redis;
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics::gauge;
use shared::{MatchEvent};
use tower_http::{trace::TraceLayer};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let state = Arc::new(AppState::new());
    let recorder=PrometheusBuilder::new().build_recorder();
    let handle=recorder.handle();
    let handle_clone=handle.clone();
        tokio::spawn(async {
        let mut sys = System::new_all();
        loop {
            sys.refresh_memory();
            let mem = sys.used_memory() as f64; // Bytes
            gauge!("memory_usage_bytes", mem);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/metrics", get(move || async move {
           handle_clone.render().into_response()
        }))
        .layer(TraceLayer::new_for_http())
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
            drop(conn);

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
