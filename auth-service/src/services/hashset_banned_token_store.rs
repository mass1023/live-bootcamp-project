use std::collections::HashSet;
use crate::domain::{TokenStoreError, BannedTokenStore};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>,
}

impl BannedTokenStore for HashsetBannedTokenStore {
    fn add_token(&mut self, token: String) -> Result<(), TokenStoreError> {
        if self.tokens.contains(token.as_str()) {
            return Err(TokenStoreError::AlreadyExists);
        }

        self.tokens.insert(token);
        Ok(())
    }

    fn token_exists(&self, token: &str) -> Result<bool, TokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_add_token_new() {
        let mut store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        let result = store.add_token("token1".to_string());
        assert!(result.is_ok());
        assert!(store.tokens.contains("token1"));
    }

    #[test]
    fn test_add_token_existing() {
        let mut store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        store.add_token("token1".to_string()).unwrap();
        let result = store.add_token("token1".to_string());
        assert_eq!(result, Err(TokenStoreError::AlreadyExists));
    }

    #[test]
    fn test_token_exists_true() {
        let mut store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        store.add_token("token1".to_string()).unwrap();
        let result = store.token_exists("token1");
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_token_exists_false() {
        let store = HashsetBannedTokenStore {
            tokens: HashSet::new(),
        };
        let result = store.token_exists("token1");
        assert_eq!(result, Ok(false));
    }
}