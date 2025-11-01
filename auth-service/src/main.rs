use auth_service::{Application, app_state::{self}, services::{HashmapUserStore, HashsetBannedTokenStore, hashmap_two_fa_code_store::HashmapTwoFACodeStore, mock_email_client::MockEmailClient}, utils::constants::prod};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let banned_token_store = HashsetBannedTokenStore::default();
    let two_fa_code_store = HashmapTwoFACodeStore::default();
    let email_client = MockEmailClient::default();

    let arc_user_store = Arc::new(RwLock::new(user_store));
    let arc_banned_token_store = Arc::new(RwLock::new(banned_token_store));
    let arc_two_fa_code_store = Arc::new(RwLock::new(two_fa_code_store));
    let app_state = Arc::new(app_state::AppState::new(arc_user_store, arc_banned_token_store, arc_two_fa_code_store, email_client));
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");
    app.run().await.expect("Failed to run application");
}
