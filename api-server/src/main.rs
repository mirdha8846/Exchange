pub mod models;

mod utils;
mod routes;
mod config;
mod middleware;
use std::sync::Arc;
use axum::{
    middleware::from_fn, response::IntoResponse, routing::{get,post}, Router
};
use sysinfo::{System};
use routes::{login_handler,order_handler};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use config::get_client;
use metrics_exporter_prometheus::{PrometheusBuilder};
use metrics::{gauge};
use crate::middleware::{auth_middleware};

#[tokio::main]
async fn main(){
    // Initialize tracing
    tracing_subscriber::fmt::init();
    let recorder=PrometheusBuilder::new().build_recorder();
    let handle=recorder.handle();
           tokio::spawn(async {
        let mut sys = System::new_all();
        loop {
            sys.refresh_memory();
            let mem = sys.used_memory() as f64; // Bytes
            gauge!("memory_usage_bytes", mem);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
     //clone handle to pass in route
     let handle_clone=handle.clone();
    // Initialize Redis connection
    let redis_conn = get_client().await;
    let shared_redis = Arc::new(tokio::sync::Mutex::new(redis_conn));
    let def_middleware= ServiceBuilder::new()
                .layer(TraceLayer::new_for_http());
    // Routes that don't need authentication
    let public_routes = Router::new()
        .route("/login", post(login_handler));

    // Routes that need authentication
    let protected_routes = Router::new()
        .route("/api/v1/order", post(order_handler))
        .layer(from_fn(auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route_layer(def_middleware)
        .route("/metrics", get(move || async move {
           handle_clone.render().into_response()
        }))
        .with_state(shared_redis);
        

    let tcplister = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Server running on http://0.0.0.0:3001");
    axum::serve(tcplister, app).await.unwrap();
}