pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug)]
pub enum LoginApiError {
    InvalidCredentials,
    UnexpectedError,
    IncorrectCredentials
}