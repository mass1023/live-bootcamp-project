use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum::extract::State;
use std::sync::Arc;


use crate::app_state::AppState;
use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME}
};

pub async fn logout(State(state): State<Arc<AppState>>, jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(c) => c,
        None => {
            return (jar, Err(AuthAPIError::MissingToken));
        }
    };
    let token = cookie.value().to_owned();
    let mut banned_token_store = state.banned_token_store.write().await;

    match validate_token(&token).await {
        Ok(_) => { 
            banned_token_store.add_token(token).expect("Error adding token to banned token store");
        },
        Err(_) => {
            return (jar, Err(AuthAPIError::InvalidToken));
        }
    }
    let jar = jar.remove(JWT_COOKIE_NAME);

    (jar, Ok(StatusCode::OK))
}