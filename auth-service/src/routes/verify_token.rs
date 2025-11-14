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

    match validate_token(&request.token, state.banned_token_store.clone()).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String
}