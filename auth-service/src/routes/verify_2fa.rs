use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, data_stores::TwoFACode, data_stores::LoginAttemptId},
    utils::auth::generate_auth_cookie,
};

pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(val) => val,
        Err(_) => {
            return (jar, Err(AuthAPIError::InvalidCredentials));
        }
    }; // Validate the email in `request`

    let login_attempt_id = match LoginAttemptId::parse(request.loging_attempt_id){
        Ok(val) => val,
        Err(_)=> {
            return (jar, Err(AuthAPIError::InvalidCredentials));
        }
    }; // Validate the login attempt ID in `request`

    let two_fa_code = match TwoFACode::parse(request.two_fa_code) {
        Ok(val) => val,
        Err(_) => {
            return (jar, Err(AuthAPIError::InvalidCredentials));
        }
    }; // Validate the 2FA code in `request`

    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    let code_tuple = match two_fa_code_store.get_code(&email).await {
        Ok(val) => val,
        Err(_) => {
            return (jar, Err(AuthAPIError::IncorrectCredentials));
        }
    };

    if login_attempt_id != code_tuple.0 {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    if two_fa_code != code_tuple.1 {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    // Generate JWT auth cookie
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => {
            return (jar, Err(AuthAPIError::UnexpectedError));
        }
    };
    let updated_jar = jar.add(auth_cookie);

    if let Err(_) = two_fa_code_store.remove_code(&email).await {
        return (updated_jar, Err(AuthAPIError::UnexpectedError));
    }

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest{
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub loging_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String
}