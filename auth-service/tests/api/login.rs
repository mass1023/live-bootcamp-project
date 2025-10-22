use crate::helpers::TestApp;

#[tokio::test]
async fn login_returns_login_ui(){
    let app = TestApp::new().await;
    let body = serde_json::json!({"email": "test@test.com", "password": "12345"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), 200);
}

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