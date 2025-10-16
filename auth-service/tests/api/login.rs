use crate::helpers::TestApp;

#[tokio::test]
async fn login_returns_login_ui(){
    let app = TestApp::new().await;

    let response = app.post_login().await;
    assert_eq!(response.status(), 200);
}