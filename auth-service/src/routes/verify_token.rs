use axum::response::IntoResponse;
use axum::http::{StatusCode};
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use std::sync::Arc;
use crate::utils::auth::validate_token;
use crate::app_state::AppState;

pub async fn verify_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyTokenRequest>
) -> impl IntoResponse {

    match validate_token(&request.token).await {
        Ok(_) => {
            // Check if token is banned
            let store = state.banned_token_store.read().await;
            match store.token_exists(&request.token) {
                Ok(true) => StatusCode::UNAUTHORIZED.into_response(),
                Ok(false) => StatusCode::OK.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        },
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
            _ => StatusCode::UNAUTHORIZED.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String
}