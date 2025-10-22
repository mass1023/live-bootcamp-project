pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
}

pub enum LoginApiError {
    InvalidCredentials,
    UnexpectedError,
    IncorrectCredentials
}