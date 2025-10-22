use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::domain::UserStoreError as ErrorUser;
use crate::domain::{LoginApiError, Email, Password};

#[axum::debug_handler]
pub async fn login(State(state): State<Arc<AppState>>, Json(request): Json<LoginRequest>) -> Result<impl IntoResponse, LoginApiError>{
    let email = Email::parse(request.email).map_err(|_| LoginApiError::InvalidCredentials)?;
    let password = Password::parse(request.password).map_err(|_| LoginApiError::InvalidCredentials)?;

    let user_store = state.user_store.read().await;

    let result = user_store.validate_user(email.as_ref(), password.as_ref()).await;

    match result {
        Ok(_) => {
            return Ok(StatusCode::OK);
        },
        Err(e) => {
            if e == ErrorUser::InvalidCredentials {
                return Err(LoginApiError::IncorrectCredentials);
            }
            return Err(LoginApiError::UnexpectedError);
        }
    }
}

#[derive(Deserialize)]
pub struct LoginRequest{
    pub email: String,
    pub password: String
}