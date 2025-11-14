use axum::response::IntoResponse;
use axum::http::{StatusCode};
use axum::Json;
use axum::extract::State;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::domain::{AuthAPIError, Email, Password};
use crate::{domain::User, app_state::AppState};
use crate::domain::UserStoreError as ErrorUser;

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(State(state): State<Arc<AppState>>,  Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError>{
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password = Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    
    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;
    
    let result = user_store.add_user(user).await;

    match result {
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully".to_string()
            });
            return Ok((StatusCode::CREATED, response));
        },
        Err(e) => {
            if e == ErrorUser::UserAlreadyExists {
                return Err(AuthAPIError::UserAlreadyExists);
            } else {
                return Err(AuthAPIError::UnexpectedError(color_eyre::eyre::eyre!("User store error: {:?}", e)));
            }
        }
        
    }
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignupResponse {
    pub message: String,
}