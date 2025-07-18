pub mod models;
mod queue;
mod utils;
mod routes;
mod middleware;
use axum::{
    middleware::from_fn, routing::{get,post}, Router
};
use middleware::auth_middleware;
use routes::{login_handler};
use tower::{ServiceBuilder};
use tower_http::{trace::TraceLayer};
#[tokio::main]
async fn main(){
let middlwears=ServiceBuilder::new().check_clone()
                                                            .layer(TraceLayer::new_for_http());
                                                            
let app=Router::new().route("/login", post(login_handler));
                            //  .layer(from_fn(auth_middleware));
let tcplister=tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
axum::serve(tcplister, app).await.unwrap();
}