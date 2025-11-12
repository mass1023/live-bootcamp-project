use std::fmt;

#[derive(Debug)]
pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
    IncorrectCredentials,
    MissingToken,
    InvalidToken
}

impl fmt::Display for AuthAPIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthAPIError::UserAlreadyExists => write!(f, "User already exists"),
            AuthAPIError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthAPIError::UnexpectedError => write!(f, "Unexpected error"),
            AuthAPIError::IncorrectCredentials => write!(f, "Incorrect credentials"),
            AuthAPIError::MissingToken => write!(f, "Missing token"),
            AuthAPIError::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

impl std::error::Error for AuthAPIError {}