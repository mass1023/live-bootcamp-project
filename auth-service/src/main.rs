use auth_service::{app_state, services::{HashmapUserStore, HashsetBannedTokenStore}, utils::constants::prod, Application};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let banned_token_store = HashsetBannedTokenStore::default();

    let arc_user_store = Arc::new(RwLock::new(user_store));
    let arc_banned_token_store = Arc::new(RwLock::new(banned_token_store));

    let app_state = Arc::new(app_state::AppState::new(arc_user_store, arc_banned_token_store));
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");
    app.run().await.expect("Failed to run application");
}
