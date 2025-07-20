use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, Query, State},
    response::IntoResponse,
};
use dashmap::DashMap;
// use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc::UnboundedSender;
use crate::types::{MatchEvent, IncomingMessage};

pub type Tx = UnboundedSender<Message>;
pub type UserId = String;
pub type Market = String;



