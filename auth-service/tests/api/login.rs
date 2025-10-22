use auth_service::ErrorResponse;

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

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&body).await;
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