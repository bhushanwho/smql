use crate::{Error, Message, MessageService};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use skyak_axum_core::errors::ApiError;
use skyak_axum_core::https::{error, success, ApiResponse};
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize, Deserialize, Debug)]
pub struct AddMessageRequest {
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetMessagesRequest {
    pub count: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteMessagesRequest {
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetryMessagesRequest {
    pub ids: Vec<String>,
}

pub async fn check() -> ApiResponse<String> {
    success("Hello World".to_string())
}

pub async fn add_message(
    State(service): State<MessageService>,
    Json(request): Json<AddMessageRequest>,
) -> ApiResponse<Message> {
    match service.add(request.body).await {
        Ok(message) => success(message),
        Err(e) => match e {
            Error::BodyTooLarge => error(ApiError::BadRequest(Some(
                "Message body size is too large".to_string(),
            ))),
            Error::Store(message) => error(ApiError::BadRequest(Some(message))),
            _ => error(ApiError::InternalServerError(Some("Internal server error".to_string()))),
        },
    }
}

pub async fn get_messages(
    State(service): State<MessageService>,
    Json(request): Json<GetMessagesRequest>,
) -> ApiResponse<Vec<Message>> {
    let count = request.count.unwrap_or(1);
    match service.get(count).await {
        Ok(messages) => success(messages),
        Err(e) => match e {
            Error::Store(message) => error(ApiError::BadRequest(Some(message))),
            _ => error(ApiError::InternalServerError(Some("Internal server error".to_string()))),
        },
    }
}

pub async fn delete_messages(
    State(service): State<MessageService>,
    Json(request): Json<DeleteMessagesRequest>,
) -> ApiResponse<String> {
    let ids = request.ids;
    match service.delete(ids).await {
        Ok(_) => success("Success".to_string()),
        Err(e) => match e {
            Error::NoIds => error(ApiError::BadRequest(Some("No message IDs provided".to_string()))),
            Error::InvalidId(id) => {
                error(ApiError::BadRequest(Some(format!("Invalid message ID: {id}"))))
            }
            Error::Store(message) => error(ApiError::BadRequest(Some(message))),
            _ => error(ApiError::InternalServerError(Some("Internal server error".to_string()))),
        },
    }
}

pub async fn purge_messages(State(service): State<MessageService>) -> ApiResponse<String> {
    match service.purge().await {
        Ok(_) => success("Success".to_string()),
        Err(e) => match e {
            Error::Store(message) => error(ApiError::BadRequest(Some(message))),
            _ => error(ApiError::InternalServerError(Some("Internal server error".to_string()))),
        },
    }
}

pub async fn retry_messages(
    State(service): State<MessageService>,
    Json(request): Json<RetryMessagesRequest>,
) -> ApiResponse<String> {
    let ids = request.ids;
    match service.retry(ids).await {
        Ok(_) => success("Success".to_string()),
        Err(e) => match e {
            Error::NoIds => error(ApiError::BadRequest(Some("No message IDs provided".to_string()))),
            Error::InvalidId(id) => {
                error(ApiError::BadRequest(Some(format!("Invalid message ID: {id}"))))
            }
            Error::Store(message) => error(ApiError::BadRequest(Some(message))),
            _ => error(ApiError::InternalServerError(Some("Internal server error".to_string()))),
        },
    }
}

pub async fn peek_messages(
    State(service): State<MessageService>,
    Json(request): Json<GetMessagesRequest>,
) -> ApiResponse<Vec<Message>> {
    let count = request.count.unwrap_or(1);
    match service.peek(count).await {
        Ok(messages) => success(messages),
        Err(e) => match e {
            Error::Store(message) => error(ApiError::BadRequest(Some(message))),
            _ => error(ApiError::InternalServerError(Some("Internal server error".to_string()))),
        },
    }
}

pub fn create_api(service: MessageService) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/hello", get(check))
        .route("/add", post(add_message))
        .route("/get", post(get_messages))
        .route("/delete", post(delete_messages))
        .route("/purge", post(purge_messages))
        .route("/retry", post(retry_messages))
        .route("/peek", post(peek_messages))
        .with_state(service)
        .layer(cors)
}