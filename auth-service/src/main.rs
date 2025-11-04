use auth_service::{
    Application, 
    app_state::{self}, 
    services::{
        HashmapUserStore, 
        HashsetBannedTokenStore, 
        hashmap_two_fa_code_store::HashmapTwoFACodeStore, 
        mock_email_client::MockEmailClient
    }, 
    utils::constants::prod,
    get_postgres_pool,
    utils::constants::DATABASE_URL
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default();
    let banned_token_store = HashsetBannedTokenStore::default();
    let two_fa_code_store = HashmapTwoFACodeStore::default();
    let email_client = MockEmailClient::default();

    let pg_pool = configure_postgresql().await;

    let arc_user_store = Arc::new(RwLock::new(user_store));
    let arc_banned_token_store = Arc::new(RwLock::new(banned_token_store));
    let arc_two_fa_code_store = Arc::new(RwLock::new(two_fa_code_store));
    let app_state = Arc::new(app_state::AppState::new(arc_user_store, arc_banned_token_store, arc_two_fa_code_store, email_client));
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build application");
    app.run().await.expect("Failed to run application");
}

async fn configure_postgresql() -> PgPool {
    println!("Configuring Postgres database... {:?}", *DATABASE_URL);
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database! 
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}