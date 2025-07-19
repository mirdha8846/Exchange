mod events;
mod config;
mod orderbook;
use std::sync::Arc;

use config::get_clinet;
#[tokio::main]
async fn main(){
    let mut conn=get_clinet().await;
    // let shared_conn=Arc::new(tokio::sync::Mutex::new(conn));
   loop{
      //get data from queue and bakki ka logic....
      let data:Option<(String,String)>=redis::cmd("BRPOP") .arg("order-queue")
            .arg(0) // block indefinitely
            .query_async(&mut conn)
            .await
            .ok();
   }
    
}

