use crate::helpers::TestApp;

#[tokio::test]
async fn logout_returns_logout_ui(){
    let app = TestApp::new().await;

    let response = app.post_logout().await;
    assert_eq!(response.status(), 200);
}