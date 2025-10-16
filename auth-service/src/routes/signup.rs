use axum::response::IntoResponse;
use axum::http::{StatusCode};
use axum::Json;
use axum::extract::State;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::domain::AuthAPIError;
use crate::{domain::User, app_state::AppState};
use crate::domain::UserStoreError as ErrorUser;
use regex::Regex;

pub async fn signup(State(state): State<Arc<AppState>>,  Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError>{
    if !is_valid_email(request.email.as_str()) {
        return Err(AuthAPIError::InvalidCredentials);
    }
    if !is_valid_password(request.password.as_str()) {
        return Err(AuthAPIError::InvalidCredentials);
    }
    
    let user = User::new(request.email, request.password, request.requires_2fa);
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
                return Err(AuthAPIError::UnexpectedError);
            }
        }
        
    }
}

fn is_valid_email(email: &str) -> bool {
    if email.is_empty() {
        return false;
    }
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

fn is_valid_password(password: &str) -> bool {
    password.len() >= 8
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("john.doe@company.co.uk"));
        assert!(is_valid_email("user+tag@example.com"));
        assert!(is_valid_email("user_name@example.org"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@domain"));
        assert!(!is_valid_email("user name@example.com"));
        assert!(!is_valid_email(""));
    }

    #[test]
    fn test_valid_passwords() {
        assert!(is_valid_password("password123"));
        assert!(is_valid_password("longenough"));
        assert!(is_valid_password("12345678")); 
    }

    #[test]
    fn test_invalid_passwords() {
        assert!(!is_valid_password("short"));
        assert!(!is_valid_password("1234567"));
        assert!(!is_valid_password(""));    
    }
}