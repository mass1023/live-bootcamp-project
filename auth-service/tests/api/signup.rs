use crate::helpers::TestApp;
use auth_service::{routes::SignupResponse, ErrorResponse};

#[tokio::test]
async fn should_return_422_if_malformed_input(){
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let test_cases = [
        serde_json::json!({"email": random_email, "requires2FA": true}), // missing password
        serde_json::json!({"email": random_email, "password": "short"}), // missing requires2FA
        serde_json::json!({"password": "password123", "requires2FA": true}), // missing email
        serde_json::json!({}), // missing both email and password and requires2FA
    ];

    for test_case in test_cases {
        let response = app.post_signup(&test_case).await;
        assert_eq!(
            response.status(),
            422,
            "The API did not fail with 422 Unprocessable Entity when the payload was {}",
            test_case);
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({"email": "not-an-email", "password": "password123", "requires2FA": true}), // invalid email
        serde_json::json!({"email": "", "password": "password123", "requires2FA": true}), // empty email
        serde_json::json!({"email": "test@test.com", "password": "123", "requires2FA": true})// empty password
    ];

    for test_case in test_cases {
        let response = app.post_signup(&test_case).await;
        assert_eq!(
            response.status(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}",
            test_case);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    // First signup should be successful
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), 201);

    // Second signup with the same email should fail with 409
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), 409);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_201_if_valid_input(){
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully".to_owned()
    };

    assert_eq!(
        response.json::<SignupResponse>().await.expect("Could not deserialize response body"),
        expected_response
    );
    app.clean_up().await;
}