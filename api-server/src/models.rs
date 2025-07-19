use serde::{Serialize,Deserialize};

// Import shared types
use shared::{OrderKind, OrderType, EnrichedOrderRequest};

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
    pub kind: OrderKind,      // limit or market
    pub order_type: OrderType, // buy or sell
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