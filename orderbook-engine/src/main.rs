mod config;
mod events;
mod orderbook;
use std::sync::Arc;
use axum::{response::IntoResponse, routing::get, Router};
use config::get_clinet;
use dashmap::DashMap;
use events::publish_event;
use orderbook::OrderBook;
use shared::{EnrichedOrderRequest, MarketType};
use metrics_exporter_prometheus::PrometheusBuilder;
use sysinfo::System;
use metrics::gauge;
#[tokio::main]
async fn main() {
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
    let app=Router::new().route("/metrics", get(move || async move {
           handle_clone.render().into_response()
        }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3600").await.unwrap();
    
    let mut conn = get_clinet().await;
    //create connection for event-queue
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let redis_conn = redis_client.get_async_connection().await.unwrap();
    let shared_conn = Arc::new(tokio::sync::Mutex::new(redis_conn));

    let books: DashMap<MarketType, OrderBook> = DashMap::new();
    loop {
        //this given below funtion return Some(queue_name,and data(raw))
        let data: Option<(String, String)> = redis::cmd("BRPOP")
            .arg("order-queue")
            .arg(0) // block indefinitely
            .query_async(&mut conn)
            .await
            .ok();
        if let Some((_, data)) = data {
            if let Ok(order) = serde_json::from_str::<EnrichedOrderRequest>(&data) {
                //this books return agr koi market exsit krta h(TATA_INR) and if not then its create new
                let mut book = books
                    .entry(order.market.clone())
                    .or_insert_with(OrderBook::new);
                let events = book.match_order(order);

                for event in events {
                    publish_event(event, &shared_conn).await; // Event bhejna
                }
            }
        }
    }
    println!("Server running on http://0.0.0.0:3600");
    axum::serve(listener, app).await.unwrap();
}
