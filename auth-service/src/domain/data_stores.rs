use super::*;

#[async_trait::async_trait]
pub trait UserStore{
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError>;
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError
}

pub trait BannedTokenStore {
    fn add_token(&mut self, token: String) -> Result<(), TokenStoreError>;
    fn token_exists(&self, token: &str) -> Result<bool, TokenStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TokenStoreError {
    AlreadyExists
}