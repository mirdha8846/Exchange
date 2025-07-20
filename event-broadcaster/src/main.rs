use std::sync::Arc;

//todo - events lena and unhe websocket and pub-sub donu ko send krna according to event type
use redis;
use shared::{MatchEvent,EventType};

#[tokio::main]
async fn main(){
    //carete conntion to event queue and pub-sub
    let redis_client=redis::Client::open("redis://127.0.0.1/").unwrap();
     let redis_conn = redis_client.get_async_connection().await.unwrap();
    let  shared_conn=Arc::new(tokio::sync::Mutex::new(redis_conn));
    loop {
        let mut conn = shared_conn.lock().await;
        let data: Option<(String, String)> = redis::cmd("BRPOP")
            .arg("order-queue")
            .arg(0) // block indefinitely
            .query_async(&mut *conn)
            .await
            .ok();
        // You can use `data` here as needed
        if let Some((_,data))=data{
           if let Ok(order)=serde_json::from_str::<MatchEvent>(&data){
            //now if else
            //agr data partial h to order book+user and totaly filled h to only user
            match order.event_type {
                //this only go to api-server not on order-book
                EventType::FullFill=>{},
                //this will go to websocket and api-server
                EventType::PartialFill=>{},
                //this will only go to api-server
                EventType::MarketPartialFill=>{}
                
            }
            
           }
        }
    }
}