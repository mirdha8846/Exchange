use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, Query, State},
    response::IntoResponse,
};
use dashmap::DashMap;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use shared::{MatchEvent, IncomingMarketType, EventType, MarketType};


use metrics::{counter, histogram};

pub type Tx = UnboundedSender<Message>;
pub type UserId = String;
pub type Market = MarketType;

#[derive(Debug, Clone)]
pub struct AppState {
    pub connections: Arc<DashMap<UserId, Tx>>,               // Active users and their senders
    pub subscribers: Arc<DashMap<Market, HashSet<UserId>>>,  // Market-wise subscribers
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            connections: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match params.get("user_id") {
        Some(uid) => uid.clone(),
        None => return "Missing user_id".into_response(),
    };

    //  Metric: Count connections
    counter!("ws_connections_total", 1, "user_id" => user_id.clone());

    ws.on_upgrade(|socket| handle_socket(socket, user_id, state))
}

pub async fn handle_socket(socket: WebSocket, user_id: UserId, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let (tx, mut rx) = unbounded_channel::<Message>();
    state.connections.insert(user_id.clone(), tx);

    let state_clone = Arc::clone(&state);
    let user_id_clone = user_id.clone();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                // Metric: Count received messages
                counter!("ws_received_total", 1, "user_id" => user_id_clone.clone());

                let start = std::time::Instant::now();

                if let Ok(req) = serde_json::from_str::<IncomingMarketType>(&text) {
                    match req {
                        IncomingMarketType::SubscribeOrderbook(market) => {
                            state_clone
                                .subscribers
                                .entry(market)
                                .or_default()
                                .insert(user_id_clone.clone());
                        }
                    }
                }

                //  Metric: Time to process client message
                histogram!(
                    "ws_receive_duration_seconds",
                    start.elapsed().as_secs_f64(),
                    "user_id" => user_id_clone.clone()
                );
            }
        }
    });

    // Outgoing channel to client
    while let Some(outgoing_msg) = rx.recv().await {
        let start = std::time::Instant::now();

        //  Metric: Count sent messages
        counter!("ws_sent_total", 1, "user_id" => user_id.clone());

        if ws_sender.send(outgoing_msg).await.is_err() {
            break;
        }

        histogram!(
            "ws_send_duration_seconds",
            start.elapsed().as_secs_f64(),
            "user_id" => user_id.clone()
        );
    }

    // Cleanup
    state.connections.remove(&user_id);
    counter!("ws_disconnections_total", 1, "user_id" => user_id.clone());
}

pub async fn handle_event(event: MatchEvent, state: &Arc<AppState>) {
    let start = std::time::Instant::now();

    let msg = Message::Text(serde_json::to_string(&event).unwrap().into());

    match event.event_type {
        EventType::FullFill | EventType::MarketPartialFill => {
            if let Some(conn) = state.connections.get(&event.user_id) {
                let _ = conn.send(msg.clone());

                //  Track event delivery to user
                counter!("event_sent_total", 1, "type" => format!("{:?}", event.event_type), "target" => "user");
            }
        }

        EventType::PartialFill => {
            if let Some(conn) = state.connections.get(&event.user_id) {
                let _ = conn.send(msg.clone());

                counter!("event_sent_total", 1, "type" => "PartialFill", "target" => "user");
            }

            if let Some(user_ids) = state.subscribers.get(&event.market) {
                for user_id in user_ids.iter() {
                    if let Some(conn) = state.connections.get(user_id) {
                        let _ = conn.send(msg.clone());

                        counter!("event_sent_total", 1, "type" => "PartialFill", "target" => "subscriber");
                    }
                }
            }
        }
    }

    histogram!(
        "event_dispatch_duration_seconds",
        start.elapsed().as_secs_f64(),
        "event_type" => format!("{:?}", event.event_type),
        "market" => format!("{:?}", event.market)
    );
}
