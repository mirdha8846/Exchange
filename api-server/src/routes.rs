use std::env;
use axum::{http::response, Extension, Json};
use dotenv::dotenv;
use crate::models::Status;

use super::models::{
    LoginResponse,IncomingLoginRequest,IncomingOrderRequest,
    ErrorResponse,Claim,OrderResponse,
};
use jsonwebtoken::{
    encode,EncodingKey, Header
};
use chrono::{Utc,Duration};

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

// pub async fn order_hanlder(Json(payload):Json<IncomingOrderRequest>,Extension(email):Extension<String>)->
// Result<Json<OrderResponse>,Json<ErrorResponse>>{
//   let email=email;
//   //what the next
//   Ok(())
// }