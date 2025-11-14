use rand::Rng;
use uuid::Uuid;
use thiserror::Error;
use color_eyre::eyre::{eyre, Context, Report, Result};

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

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Already exists error")]
    AlreadyExists,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for BannedTokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::AlreadyExists, Self::AlreadyExists)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report)
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self> {
        let parsed_id = uuid::Uuid::parse_str(&id).wrap_err("Invalid login attempt id")?;
        Ok(Self(parsed_id.to_string()))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let uuid = Uuid::new_v4();
        LoginAttemptId(uuid.to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self> {
        let code_as_u32 = code.parse::<u32>().wrap_err("Invalid 2FA code")?;

        if (100_000..=999_999).contains(&code_as_u32) {
            Ok(Self(code))
        } else {
            Err(eyre!("Invalid 2FA code")) 
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let six_digit_number = rng.random_range(100000..1000000);
        TwoFACode(six_digit_number.to_string())
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}