pub mod models;
mod queue;
mod utils;
mod routes;
mod middleware;
use axum::{
    routing::{get,post},
    Router
};
use routes::{login_handler};

#[tokio::main]
async fn main(){
let app=Router::new().route("/login", post(login_handler));
let tcplister=tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
axum::serve(tcplister, app);
}