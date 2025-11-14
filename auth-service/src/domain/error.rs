use thiserror::Error;
use color_eyre::eyre::Report;

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken
}

