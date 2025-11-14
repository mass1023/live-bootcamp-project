use std::sync::Arc;

use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token);
        let ttl = TOKEN_TTL_SECONDS as u64;
        let mut conn = self.conn.write().await;
        conn.set_ex(&key, true, ttl)
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let mut conn = self.conn.write().await;
        let is_banned = conn.exists(&key)
            .wrap_err("failed to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(is_banned)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}