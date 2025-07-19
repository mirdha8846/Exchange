use std::{env, sync::Arc};
use axum::{extract::State, http::response, Extension, Json};
use dotenv::dotenv;
use redis::AsyncCommands;
use crate::models::{EnrichedOrderRequest, Status};

use super::models::{
    LoginResponse,IncomingLoginRequest,IncomingOrderRequest,
    ErrorResponse,Claim,OrderResponse,
};
use jsonwebtoken::{
    encode,EncodingKey, Header
};
use chrono::{Utc,Duration};
use uuid::Uuid;
pub async fn login_handler(Json(payload):Json<IncomingLoginRequest>)->Result<Json<LoginResponse>,Json<ErrorResponse>>{
   dotenv().ok();
   let email=payload.email;
   let secret=env::var("JWT_SECRET").unwrap();
   let claims=Claim{
    email:email,
    exp:(Utc::now()+Duration::hours(5)).timestamp() as usize
   };
   let token=encode(&Header::default(),&claims ,&EncodingKey::from_secret(secret.as_ref()));
   match token {
       Ok(value)=>{
        let response=LoginResponse{
            status:Status::Success,
            message:"login successful".to_string(),
            token:value
        };
        return Ok(Json::from(response));
       },
       Err(e)=>{
        let response=ErrorResponse{
            status:Status::Error,
            error:e.to_string()
        };
        return Err(Json::from(response));
       }
   }

} 

pub async fn order_handler(
    State(shared_redis): State<Arc<tokio::sync::Mutex<redis::aio::Connection>>>,
    Extension(email): Extension<String>,
    Json(payload): Json<IncomingOrderRequest>
) -> Result<Json<OrderResponse>, Json<ErrorResponse>>{
  let email=email;
  //what the next
  let order_id=Uuid::new_v4().to_string();
  let order=EnrichedOrderRequest{
   user_id:email,
   order_id:order_id.clone(),
   kind:payload.kind,
   order_type:payload.order_type,
   price:payload.price,
   quantity:payload.quantity,
   market:payload.market
  };
  let mut conn=shared_redis.lock().await;
  let messg=serde_json::to_string(&order).unwrap();
  if let Err(e) = conn.lpush::<_, _, ()>("order-queue", messg).await {
      let response = ErrorResponse {
          status: Status::Error,
          error: e.to_string(),
      };
      return Err(Json::from(response));
  }
  //send to queue and notify user
 let response=OrderResponse{
     status:Status::Success,
    order_id:order_id
 };

  Ok(Json::from(response))
}