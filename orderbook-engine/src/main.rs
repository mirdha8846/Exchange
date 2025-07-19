mod events;
mod config;
mod orderbook;
use dashmap::DashMap;
use events::publish_event;
use orderbook::OrderBook;
// use std::sync::Arc;
use shared::{EnrichedOrderRequest,MarketType};
use config::get_clinet;
#[tokio::main]
async fn main(){
    let mut conn=get_clinet().await;
    // let shared_conn=Arc::new(tokio::sync::Mutex::new(conn));
      let books: DashMap<MarketType, OrderBook> = DashMap::new();
   loop{
      //get data from queue and bakki ka logic....
      let data:Option<(String,String)>=redis::cmd("BRPOP") .arg("order-queue")
            .arg(0) // block indefinitely
            .query_async(&mut conn)
            .await
            .ok();
       if let Some((queue,raw))=data{
          if let Ok(order)=serde_json::from_str::<EnrichedOrderRequest>(&raw){
           let mut book=books.entry(order.market.clone()).or_insert_with(OrderBook::new);
           let events=book.match_order(order);
           
                for event in events {
                    publish_event(event).await; // Event bhejna
                }
          }
        }
   
        
    }
    
    
}


