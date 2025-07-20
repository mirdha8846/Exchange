mod events;
mod config;
mod orderbook;
use std::sync::{Arc};

use dashmap::DashMap;
use events::publish_event;
use orderbook::OrderBook;
use shared::{EnrichedOrderRequest,MarketType};
use config::get_clinet;
#[tokio::main]
async fn main(){
    let mut conn=get_clinet().await;
    //create connection for event-queue
let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
let redis_conn = redis_client.get_async_connection().await.unwrap();
let shared_conn =Arc::new(tokio::sync::Mutex::new(redis_conn));
    
    let books: DashMap<MarketType, OrderBook> = DashMap::new();
   loop{
      //this given below funtion return Some(queue_name,and data(raw))
      let data:Option<(String,String)>=redis::cmd("BRPOP") .arg("order-queue")
            .arg(0) // block indefinitely
            .query_async(&mut conn)
            .await
            .ok();
       if let Some((_,data))=data{
          if let Ok(order)=serde_json::from_str::<EnrichedOrderRequest>(&data){
            //this books return agr koi market exsit krta h(TATA_INR) and if not then its create new
           let mut book=books.entry(order.market.clone()).or_insert_with(OrderBook::new);
           let events=book.match_order(order);
           
                for event in events {
                     publish_event(event, &shared_conn).await; // Event bhejna
                }
          }
        }
    }
    
}


