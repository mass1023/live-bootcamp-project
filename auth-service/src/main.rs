use auth_service::{
    Application, app_state::{self}, get_postgres_pool, get_redis_client, services::{
        RedisBannedTokenStore, RedisTwoFACodeStore, data_stores::PostgresUserStore, mock_email_client::MockEmailClient
    }, utils::{constants::{DATABASE_URL, REDIS_HOST_NAME, prod}, tracing::init_tracing}
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    println!("=== APPLICATION STARTING ===");
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");
    
    let pg_pool = configure_postgresql().await;
    let redis_conn = configure_redis();
    let arc_redis_conn = Arc::new(RwLock::new(redis_conn));
    let user_store = PostgresUserStore::new(pg_pool);
    let banned_token_store = RedisBannedTokenStore::new(arc_redis_conn.clone());
    let two_fa_code_store = RedisTwoFACodeStore::new(arc_redis_conn);
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

async fn configure_postgresql() -> PgPool {
    println!("Configuring Postgres database... {:?}", *DATABASE_URL);
    
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");
    
    println!("Running migrations...");
    match sqlx::migrate!("./migrations").run(&pg_pool).await {
        Ok(_) => println!("Migrations completed successfully!"),
        Err(e) => {
            eprintln!("Migration error: {:?}", e);
            panic!("Failed to run migrations: {}", e);
        }
    }

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}