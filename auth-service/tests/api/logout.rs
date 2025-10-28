use crate::helpers::TestApp;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie(){
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
    let token = auth_cookie.value();
    let store = app.banned_token_store.write().await;
    let response = store.token_exists(token);
    assert_eq!(response, Ok(true));
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
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

    let response = app.post_logout().await;
    assert_eq!(response.status(), 400);
}