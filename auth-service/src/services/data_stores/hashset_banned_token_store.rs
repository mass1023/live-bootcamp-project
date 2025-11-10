use std::collections::HashSet;
use crate::domain::{BannedTokenStoreError, BannedTokenStore};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if self.tokens.contains(token.as_str()) {
            return Err(BannedTokenStoreError::AlreadyExists);
        }

        self.tokens.insert(token);
        Ok(())
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_add_token_new() {
        let mut store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        let result = store.add_token("token1".to_string()).await;
        assert!(result.is_ok());
        assert!(store.tokens.contains("token1"));
    }

    #[tokio::test]
    async fn test_add_token_existing() {
        let mut store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        store.add_token("token1".to_string()).await.unwrap();
        let result = store.add_token("token1".to_string()).await;
        assert_eq!(result, Err(BannedTokenStoreError::AlreadyExists));
    }

    #[tokio::test]
    async fn test_contains_token_true() {
        let mut store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        store.add_token("token1".to_string()).await.unwrap();
        let result = store.contains_token("token1").await;
        assert_eq!(result, Ok(true));
    }

    #[tokio::test]
    async fn test_contains_token_false() {
        let store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        let result = store.contains_token("token1").await;
        assert_eq!(result, Ok(false));
    }
}