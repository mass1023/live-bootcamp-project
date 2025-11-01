use auth_service::{domain::Email, utils::{auth::generate_auth_cookie, constants::JWT_COOKIE_NAME}};
use reqwest::Url;
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input(){
    let app = TestApp::new().await;
    let malformed_jwt = serde_json::json!({
        "token": "test"
    });

    let response = app.post_verify_token(&malformed_jwt).await;
    assert_eq!(response.status(), 422);   
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let email = Email::parse(random_email).expect("Failed to parse random email");
    let jwt = generate_auth_cookie(&email).expect("Failed to generate auth cookie");
    let body = serde_json::json!({
        "token": jwt.value()
    });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;
    let invalid_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0QGV4YW1wbGUuY29tIiwiZXhwIjoxNjg0ODk5MjAwfQ.invalid_signature";
    let body = serde_json::json!({
        "token": invalid_token
    });

    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    let login_body = serde_json::json!({"email": random_email, "password": "password123"});

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let token = auth_cookie.value();
    let body = serde_json::json!({"token": token});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), 200);

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME,
            auth_cookie.value()
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status(), 200);
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), 401);
}