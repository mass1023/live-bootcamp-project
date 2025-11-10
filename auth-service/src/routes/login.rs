use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domain::{EmailClient, UserStoreError as ErrorUser};
use crate::domain::{AuthAPIError, Email, Password, data_stores::{LoginAttemptId, TwoFACode}};
use crate::utils::auth::generate_auth_cookie;

pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<LoginResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(res) => res,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };
    let password = match Password::parse(request.password) {
        Ok(res)=> res,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let user_store = state.user_store.read().await;

    let result = user_store.validate_user(email.as_ref(), password.as_ref()).await;

    match result {
        Ok(_) => {
            let user = match user_store.get_user(email.as_ref()).await {
                Ok(user) => user,
                Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials))
            };

            let result = match user.requires_2fa {
                true => handle_2fa(&user.email, &state, jar).await,
                false => handle_no_2fa(&user.email, jar).await
            };

            return result;
        },
        Err(e) => {
            if e == ErrorUser::InvalidCredentials {
                return (jar, Err(AuthAPIError::IncorrectCredentials));
            }
            return (jar, Err(AuthAPIError::UnexpectedError));
        }
    }
}

async fn handle_2fa (email: &Email, state: &AppState, jar: CookieJar)
    -> (CookieJar, Result<LoginResponse, AuthAPIError>) {
    // First, we must generate a new random login attempt ID and 2FA code
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    // Store the ID and code in our 2FA code store. Return `AuthAPIError::UnexpectedError` if the operation fails
    let mut two_fa_store = state.two_fa_code_store.write().await;
    if let Err(_) = two_fa_store.add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone()).await {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    if let Err(_) = state.email_client.send_email(&email, "2FA Code", &two_fa_code.as_ref()).await {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    let response = TwoFactorAuthResponse{
        message: "2FA Required".to_string(),
        loging_attempt_id: login_attempt_id.as_ref().to_string()
    };
    return (jar, Ok(LoginResponse::TwoFactorAuth(response)));
}

async fn handle_no_2fa(email: &Email, jar: CookieJar)
    -> (CookieJar, Result<LoginResponse, AuthAPIError>) {
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(res)=> res,
        Err(_) => {
            return (jar, Err(AuthAPIError::UnexpectedError));
        }
    };
    let updated_jar = jar.add(auth_cookie);

    // For non-2FA logins, we still return a TwoFactorAuthResponse but with empty loginAttemptId
    // This is for consistency with the API response format
    let response = TwoFactorAuthResponse{
        message: "Login successful".to_string(),
        loging_attempt_id: String::new()
    };
    return (updated_jar, Ok(LoginResponse::TwoFactorAuth(response)));
}

#[derive(Deserialize)]
pub struct LoginRequest{
    pub email: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub loging_attempt_id: String
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse)
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> Response {
        match self {
            LoginResponse::RegularAuth => StatusCode::OK.into_response(),
            LoginResponse::TwoFactorAuth(response) => (StatusCode::OK, Json(response)).into_response()
        }
    }
}
