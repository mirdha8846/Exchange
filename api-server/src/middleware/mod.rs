use axum::{
    body::Body, extract::Request, http::StatusCode, middleware::Next, response::{IntoResponse, Response}, Json
};
use dotenv::dotenv;
use jsonwebtoken::{
    Validation,DecodingKey,decode
};
use std::env;

use crate::models::{Claim,ErrorResponse,Status};

pub async fn auth_middleware(mut req:Request<Body>,next:Next)->Result<Response,StatusCode>{
    dotenv().ok();
    let secret=env::var("JWT_SECRET").unwrap();
    let header=req.headers();
    if let Some(auth_header)=header.get("Authorization"){
     if let Ok(auth_header_str)=auth_header.to_str(){
        if auth_header_str.starts_with("Bearer "){
            let token=&auth_header_str[7..];
            let decoded_token=decode::<Claim>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default());
          match decoded_token {
              Ok(result)=>{
                let email=result.claims.email;
                req.extensions_mut().insert(email);
                let response = next.run(req).await;
            return Ok(response);
              },
              Err(_)=>{
              
               return Err(StatusCode::UNAUTHORIZED);
              }
          }
        }
     }
    }
     Err(StatusCode::UNAUTHORIZED)
   
} 