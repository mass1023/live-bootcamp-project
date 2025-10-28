use uuid::Uuid;
use auth_service::{app_state::{AppState, BannedTokenStoreType}, services::{HashmapUserStore, HashsetBannedTokenStore}, utils::constants::test, Application};
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::cookie::Jar;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub cookie_jar: Arc<Jar>
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::default();
        let banned_token_store = HashsetBannedTokenStore::default();
        let arc_user_store = Arc::new(RwLock::new(user_store));
        let arc_banned_token_store = Arc::new(RwLock::new(banned_token_store));
        let app_state = Arc::new(AppState::new(arc_user_store, arc_banned_token_store.clone()));
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

        Self { address, http_client, banned_token_store: arc_banned_token_store, cookie_jar }
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

    pub async fn post_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
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

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}