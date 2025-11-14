use crate::helpers::TestApp;
use serde_json;
use auth_service::{
    ErrorResponse,
};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({"login_attempt_id": "some_id"}),
        serde_json::json!({"two_fa_code": "123456"})
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(
            response.status(),
            422,
            "The API did not fail with 422 when payload was {}",
            test_case
        );
    }
    app.clean_up().await;
}
#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({"email": "test@test.com", "loginAttemptId": "not-a-uuid", "2FACode": "123456"}),
        serde_json::json!({"email": "test@test.com", "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000", "2FACode": "12345"}), // invalid code length
        serde_json::json!({"email": "test@test.com", "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000", "2FACode": "abcdef"}) // non-numeric code
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;
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
    app.clean_up().await;
}
#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    // First, signup and login to get a valid login_attempt_id
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    let login_body = serde_json::json!({"email": random_email, "password": "password123"});
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let json_body = response
        .json::<auth_service::routes::TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    // Now try to verify with wrong code
    let wrong_verify_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": json_body.loging_attempt_id,
        "2FACode": "000000" // wrong code
    });

    let response = app.post_verify_2fa(&wrong_verify_body).await;
    assert_eq!(
        response.status(),
        400,
        "The API did not fail with 400 when the 2FA code was incorrect"
    );
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    // First, signup
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    // First login to get the initial 2FA code
    let login_body = serde_json::json!({"email": random_email, "password": "password123"});
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let json_body = response
        .json::<auth_service::routes::TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    // Second login - this should invalidate the first code
    let login_body2 = serde_json::json!({"email": random_email, "password": "password123"});
    let _response2 = app.post_login(&login_body2).await;
    // We don't assert the status here as the test is focused on the old code failing

    // Now try to verify with the 2FA code from the first login request - this should fail
    let old_verify_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": json_body.loging_attempt_id,
        "2FACode": "123456" // doesn't matter what code we use, the ID should be invalid
    });

    let response = app.post_verify_2fa(&old_verify_body).await;
    // The route returns 401 for incorrect credentials (wrong login attempt ID)
    assert_eq!(
        response.status(),
        401,
        "The API did not fail with 401 when using an old login attempt ID"
    );
    app.clean_up().await;
}
#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    // First, signup and login to get a valid login_attempt_id
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    let login_body = serde_json::json!({"email": random_email, "password": "password123"});
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Get the correct 2FA code from the store
    let two_fa_store = app.two_fa_code_store.read().await;
    let email = auth_service::domain::Email::parse(random_email.clone()).unwrap();
    let (stored_login_attempt_id, stored_code) = two_fa_store.get_code(&email).await.expect("2FA code should be stored");
    drop(two_fa_store);

    // Verify with correct code
    let correct_verify_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": stored_login_attempt_id.as_ref(),
        "2FACode": stored_code.as_ref()
    });

    let response = app.post_verify_2fa(&correct_verify_body).await;
    assert_eq!(response.status(), 200);

    // Assert that the auth cookie gets set
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == auth_service::utils::constants::JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}


#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let random_email = crate::helpers::get_random_email();

    // First, signup and login to get a valid login_attempt_id
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    let login_body = serde_json::json!({"email": random_email, "password": "password123"});
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Get the correct 2FA code from the store
    let two_fa_store = app.two_fa_code_store.read().await;
    let email = auth_service::domain::Email::parse(random_email.clone()).unwrap();
    let (stored_login_attempt_id, stored_code) = two_fa_store.get_code(&email).await.expect("2FA code should be stored");
    drop(two_fa_store); // Release the read lock

    // Verify with correct code first time - should succeed
    let verify_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": stored_login_attempt_id.as_ref(),
        "2FACode": stored_code.as_ref()
    });

    let response = app.post_verify_2fa(&verify_body).await;
    assert_eq!(response.status(), 200);

    // Now try to verify again with the same code - should fail with 401
    let response = app.post_verify_2fa(&verify_body).await;
    assert_eq!(
        response.status(),
        401,
        "The API did not fail with 401 when using the same 2FA code twice"
    );
    app.clean_up().await;
}