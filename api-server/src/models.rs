use std::error;

use axum::Error;
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub enum OrderKind {
    Limit,
    Market
}
#[derive(Serialize,Deserialize)]
pub enum OrderType {
    Buy,
   Sell
}

#[derive(Serialize,Deserialize)]
pub enum Status {
   Error,
   Success
}

//Icoming request
#[derive(Serialize,Deserialize)]
pub struct IncomingLoginRequest{
   pub name:String,
   pub email:String
}
#[derive(Serialize,Deserialize)]
pub struct IncomingOrderRequest{
    pub kind: OrderKind,      // buy or sell
    pub order_type: OrderType, // limit or market
    pub price: f64,
    pub quantity: u64,
    pub market: String,
}

// #[derive(Serialize,Deserialize)]
pub struct EnrichedOrderRequest{
    pub user_id:String,
    pub order_id:String,
    pub kind: OrderKind,      // buy or sell
    pub order_type: OrderType, // limit or market
    pub price: f64,
    pub quantity: u64,
    pub market: String,
}

//resposne

#[derive(Serialize,Deserialize)]
pub struct LoginResponse{
  pub status:Status,
  pub message:String,
  pub token:String
}

#[derive(Serialize,Deserialize)]
pub struct ErrorResponse{
  pub status:Status,
   pub error:String
}
#[derive(Serialize,Deserialize)]
pub struct OrderResponse{
   pub status:Status,
   pub order_id:String,
}

//claim for token
#[derive(Serialize,Deserialize)]
pub struct Claim{
   pub email:String,
   pub exp:usize
}