use sqlx::{Connection, Executor, PgConnection, PgPool, postgres::{PgConnectOptions, PgPoolOptions}};
use uuid::Uuid;
use auth_service::{
    Application, app_state::{
        AppState,
        BannedTokenStoreType, TwoFACodeStoreType
    }, get_postgres_pool, get_redis_client, services::{
        RedisTwoFACodeStore, RedisBannedTokenStore, data_stores::PostgresUserStore, mock_email_client::MockEmailClient
    }, utils::constants::{DATABASE_URL, REDIS_HOST_NAME, test}
};
use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use reqwest::cookie::Jar;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub cookie_jar: Arc<Jar>,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub db_name: String,
    pub cleaned_up: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (pg_pool, db_name) = configure_postgresql().await;
        let redis_conn = configure_redis();
        let arc_redis_conn = Arc::new(RwLock::new(redis_conn));
        let user_store = PostgresUserStore::new(pg_pool);
        let banned_token_store = RedisBannedTokenStore::new(arc_redis_conn.clone());
        let two_fa_code_store = RedisTwoFACodeStore::new(arc_redis_conn);
        let arc_user_store = Arc::new(RwLock::new(user_store));
        let arc_banned_token_store = Arc::new(RwLock::new(banned_token_store));
        let arc_two_fa_code_store = Arc::new(RwLock::new(two_fa_code_store));
        let email_client = MockEmailClient::default();
        let app_state = Arc::new(AppState::new(arc_user_store, arc_banned_token_store.clone(), arc_two_fa_code_store.clone(), email_client));
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());
        let cookie_jar = Arc::new(Jar::default());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self { address, http_client, banned_token_store: arc_banned_token_store, cookie_jar, two_fa_code_store: arc_two_fa_code_store, db_name, cleaned_up: false }
    }

    pub async fn clean_up(&mut self) {
        let db_name = self.db_name.clone();
        delete_database(&db_name).await;
        self.cleaned_up = true;
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response 
    where
        Body: serde::Serialize
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where Body: serde::Serialize {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) 
        -> reqwest::Response where Body: serde::Serialize {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response 
    where
        Body: serde::Serialize
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("TestApp was not cleaned up! Call clean_up() before the test ends.");
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

fn configure_redis() -> redis::Connection {
    println!("Configuring Redis... {:?}", REDIS_HOST_NAME.to_owned());
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

async fn configure_postgresql() -> (PgPool, String) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    // we need to remove the database name at the end to create a new database
    let server_url = postgresql_conn_url.rsplit_once('/').unwrap().0.to_string();

    configure_database(&server_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", server_url, db_name);

    // Create a new connection pool and return it
    let pool = get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!");

    (pool, db_name)
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");


    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}