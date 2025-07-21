use std::{env, sync::Arc, time::Instant};
use axum::{extract::{State}, Extension, Json};
use dotenv::dotenv;
use redis::AsyncCommands;
use crate::models::Status;

use shared::EnrichedOrderRequest;

use super::models::{
    LoginResponse, IncomingLoginRequest, IncomingOrderRequest,
    ErrorResponse, Claim, OrderResponse,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use uuid::Uuid;

// Metrics
use metrics::{counter, histogram};

pub async fn login_handler(
    Json(payload): Json<IncomingLoginRequest>
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    dotenv().ok();
    let start = Instant::now(); // ⏱ Start measuring

    let email = payload.email;
    let secret = env::var("JWT_SECRET").unwrap();

    // Count total login attempts
    counter!("login_attempts_total", 1, "email" => email.clone());

    let claims = Claim {
        email: email.clone(),
        exp: (Utc::now() + Duration::hours(5)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()));

    match token {
        Ok(value) => {
            counter!("login_success_total", 1, "email" => email.clone());

            histogram!("login_duration_seconds", start.elapsed().as_secs_f64(), "status" => "success");

            let response = LoginResponse {
                status: Status::Success,
                message: "login successful".to_string(),
                token: value,
            };
            Ok(Json::from(response))
        },
        Err(e) => {
            counter!("login_failure_total", 1, "email" => email.clone());

            histogram!("login_duration_seconds", start.elapsed().as_secs_f64(), "status" => "failure");

            let response = ErrorResponse {
                status: Status::Error,
                error: e.to_string(),
            };
            Err(Json::from(response))
        }
    }
}

pub async fn order_handler(
    State(shared_redis): State<Arc<tokio::sync::Mutex<redis::aio::Connection>>>,
    Extension(email): Extension<String>,
    Json(payload): Json<IncomingOrderRequest>
) -> Result<Json<OrderResponse>, Json<ErrorResponse>> {
    let start = Instant::now(); // ⏱ Start measuring

    let email = email;
    let order_id = Uuid::new_v4().to_string();

    let order = EnrichedOrderRequest {
        user_id: email.clone(),
        order_id: order_id.clone(),
        kind: payload.kind,
        order_type: payload.order_type,
        price: payload.price,
        quantity: payload.quantity,
        market: payload.market.clone(),
    };

    let mut conn = shared_redis.lock().await;
    let messg = serde_json::to_string(&order).unwrap();

    if let Err(e) = conn.lpush::<_, _, ()>("order-queue", messg).await {
        counter!("orders_failed_total", 1, "market" => format!("{:?}", order.market));

        histogram!("order_duration_seconds", start.elapsed().as_secs_f64(), "status" => "failure");

        let response = ErrorResponse {
            status: Status::Error,
            error: e.to_string(),
        };
        return Err(Json::from(response));
    }

    counter!("orders_placed_total", 1, 
        "market" => format!("{:?}", order.market),
        "kind" => format!("{:?}", order.kind),
        "order_type" => format!("{:?}", order.order_type),
    );

    histogram!("order_duration_seconds", start.elapsed().as_secs_f64(),
        "status" => "success",
        "market" => format!("{:?}", order.market),
        "kind" => format!("{:?}", order.kind),
        "order_type" => format!("{:?}", order.order_type),
    );

    let response = OrderResponse {
        status: Status::Success,
        order_id: order_id,
    };

    Ok(Json::from(response))
}
