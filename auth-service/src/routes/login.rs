use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domain::UserStoreError as ErrorUser;
use crate::domain::{LoginApiError, Email, Password};
use crate::utils::auth::generate_auth_cookie;

#[axum::debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar, // New!
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<StatusCode, LoginApiError>) {
    let email = match Email::parse(request.email) {
        Ok(res) => res,
        Err(_) => return (jar, Err(LoginApiError::InvalidCredentials))
    };
    let password = match Password::parse(request.password) {
        Ok(res)=> res,
        Err(_) => return (jar, Err(LoginApiError::InvalidCredentials))
    };

    let user_store = state.user_store.read().await;

    let result = user_store.validate_user(email.as_ref(), password.as_ref()).await;

    match result {
        Ok(_) => {
            let auth_cookie = match generate_auth_cookie(&email) {
                Ok(res)=> res,
                Err(_) => {
                    return (jar, Err(LoginApiError::UnexpectedError));
                }
            };
            let updated_jar = jar.add(auth_cookie);
            return (updated_jar, Ok(StatusCode::OK));
        },
        Err(e) => {
            if e == ErrorUser::InvalidCredentials {
                return (jar, Err(LoginApiError::IncorrectCredentials));
            }
            return (jar, Err(LoginApiError::UnexpectedError));
        }
    }
}

#[derive(Deserialize)]
pub struct LoginRequest{
    pub email: String,
    pub password: String
}