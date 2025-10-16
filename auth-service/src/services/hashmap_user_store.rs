use std::collections::HashMap;
use crate::domain::{User, UserStore, UserStoreError};



#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(user.email.0.as_str()) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email.0.clone(), user);
        Ok(())
    }

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.get_user(email).await {
            Ok(user) => {
                if user.password.as_ref() == password {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{Email, Password};

    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let user = User {
            email: Email("test@mytest.com".to_string()),
            password: Password("password123".to_string()),
            requires_2fa: false,
        };
        let mut store = HashmapUserStore::default();
        let result = store.add_user(user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let user = User {
            email: Email("test@mytest.com".to_string()),
            password: Password("password123".to_string()),
            requires_2fa: false,
        };
        let mut store = HashmapUserStore::default();
        let add_result = store.add_user(user.clone()).await;
        assert!(add_result.is_ok());
        let retrieved_user = store.get_user(&user.email.as_ref()).await;
        assert!(retrieved_user.is_ok());
        assert_eq!(retrieved_user.unwrap(), &user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user = User {
            email: Email("test@mytest.com".to_string()),
            password: Password("password123".to_string()),
            requires_2fa: false,
        };
        let mut store = HashmapUserStore::default();
        let add_result = store.add_user(user.clone()).await;
        assert!(add_result.is_ok());
        let validate_result = store.validate_user(&user.email.as_ref(), &user.password.as_ref()).await;
        assert!(validate_result.is_ok());
        let invalid_validate_result = store.validate_user(&user.email.as_ref(), "wrongpassword").await;
        assert_eq!(invalid_validate_result, Err(UserStoreError::InvalidCredentials));
        let not_found_result = store.validate_user("nonexistent@test.com", "password").await;
        assert_eq!(not_found_result, Err(UserStoreError::UserNotFound));
    }
}