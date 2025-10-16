use crate::helpers::TestApp;

#[tokio::test]
async fn verify_2fa_returns_verify_2fa_ui(){
    let app = TestApp::new().await;

    let response = app.post_verify_2fa().await;
    assert_eq!(response.status(), 200);
}