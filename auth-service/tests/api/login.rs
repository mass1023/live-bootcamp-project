use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let test_cases = [
        serde_json::json!({"email": random_email}),
        serde_json::json!({"password": "12345"})
    ];

    for test_case in test_cases{
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status(),
            422,
            "The API did not fail with 422 when payload was {}",
            test_case
         );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let test_cases = [
        serde_json::json!({"email": random_email, "password": "12345"}),
        serde_json::json!({"email": "test", "password": "12345678"})
    ];

    for test_case in test_cases{
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}", 
            test_case
        );

        assert_eq!(
            response.json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    let wrong_credentials = serde_json::json!({
        "email": random_email,
        "password": "password124"
    });

    let response = app.post_login(&wrong_credentials).await;

    assert_eq!(
        response.status(),
        401,
        "The API did not fail with 401 Invalid Credentials when the payload was {}", 
        wrong_credentials
    );
}


#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
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

    assert!(!auth_cookie.value().is_empty());
}