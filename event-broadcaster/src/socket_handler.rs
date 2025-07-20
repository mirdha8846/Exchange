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

// Type aliases for readability
pub type Tx = UnboundedSender<Message>;
pub type UserId = String;
pub type Market = MarketType;

// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    // this connections looks like this
    // "user123" => tx123
    // "user456" => tx456

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

// Entry point for WebSocket connections
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_id = match params.get("user_id") {
        Some(uid) => uid.clone(),
        None => return "Missing user_id".into_response(),
    };

    ws.on_upgrade(|socket| handle_socket(socket, user_id, state))
}

// Main logic for handling WebSocket connection
pub async fn handle_socket(socket: WebSocket, user_id: UserId, state: Arc<AppState>) {
    // Step 1: Split socket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Step 2: Create unbounded channel to send messages to this user
    // a Rust internal communication channel (not related to the WebSocket) 
    // that our server uses internally.
    let (tx, mut rx) = unbounded_channel::<Message>();

    // Step 3: Store user sender in AppState
    state.connections.insert(user_id.clone(), tx);

    // Step 4: Spawn background task to receive messages from the client
    //tokio::spawn(...) ke closure ke andar agar aap outer variable use karna
    // chahte ho to uska ownership ya clone chahiye.(means userid,state both ka clone krna pdega )
    let state_clone = Arc::clone(&state);
    let user_id_clone = user_id.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(req) = serde_json::from_str::<IncomingMarketType>(&text) {
                    match req {
                        IncomingMarketType::SubscribeOrderbook(market) => {
                            state_clone
                                .subscribers
                                .entry(market)//this one search for market
                                .or_default()//agr market nhi mila to bna dega with empty hashset
                                .insert(user_id_clone.clone());//and ye us market se userid map kr dega
                        }
                    }
                }
            }
        }
    });

    // Step 5: Forward messages from rx to WebSocket client
    //rust me js ki trh event specific nhi hota sab ko ek hi jegh send krte h jese abhi kr rhe h
    /*
    this is how we get in frontend
    socket.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.event_type === "PartialFill") {
    // Update user-specific status AND orderbook
  } else if (data.event_type === "FullFill") {
    // Notify user that order is fully filled
  } else if (data.event_type === "MarketPartialFill") {
    // Show market-specific notification to user
  }
};
     */
    while let Some(outgoing_msg) = rx.recv().await {
        if ws_sender.send(outgoing_msg).await.is_err() {
            break; // client disconnected
        }
    }

    // Step 6: Clean up on disconnect
    state.connections.remove(&user_id);
}

// Event handler that sends messages to specific users or all subscribers
pub async fn handle_event(event: MatchEvent, state: &Arc<AppState>) {
    let msg = Message::Text(serde_json::to_string(&event).unwrap().into());

    match event.event_type {
        // these two type only user specific not on
        EventType::FullFill | EventType::MarketPartialFill => {
            if let Some(conn) = state.connections.get(&event.user_id) {
                let _ = conn.send(msg.clone());
            }
        },

        // ye user specific ko bhi jayega and orderbook ko bhi jayega
        EventType::PartialFill => {
            // 1. Notify user
            if let Some(conn) = state.connections.get(&event.user_id) {
                let _ = conn.send(msg.clone());
            }

            // 2. Notify all market subscribers — **including the user also**
            if let Some(user_ids) = state.subscribers.get(&event.market) {
                for user_id in user_ids.iter() {
                    if let Some(conn) = state.connections.get(user_id) {
                        let _ = conn.send(msg.clone());
                    }
                }
            }
        }
    }
}
