use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore{
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        if self.codes.contains_key(&email) {
            return Err(TwoFACodeStoreError::UnexpectedError)
        }
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(&email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes.get(email).cloned().ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_code_success() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@test.com".to_string()).unwrap();
        let login_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        let result = store.add_code(email.clone(), login_id.clone(), code.clone()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_code_already_exists() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@test.com".to_string()).unwrap();
        let login_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), login_id.clone(), code.clone()).await.unwrap();
        let result = store.add_code(email, LoginAttemptId::default(), TwoFACode::default()).await;
        assert_eq!(result, Err(TwoFACodeStoreError::UnexpectedError));
    }

    #[tokio::test]
    async fn test_remove_code_existing() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@test.com".to_string()).unwrap();
        let login_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), login_id.clone(), code.clone()).await.unwrap();
        let result = store.remove_code(&email).await;
        assert!(result.is_ok());
        let get_result = store.get_code(&email).await;
        assert_eq!(get_result, Err(TwoFACodeStoreError::LoginAttemptIdNotFound));
    }

    #[tokio::test]
    async fn test_remove_code_non_existing() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@test.com".to_string()).unwrap();
        let result = store.remove_code(&email).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_code_existing() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@test.com".to_string()).unwrap();
        let login_id = LoginAttemptId::default();
        let code = TwoFACode::default();
        store.add_code(email.clone(), login_id.clone(), code.clone()).await.unwrap();
        let result = store.get_code(&email).await;
        assert_eq!(result, Ok((login_id, code)));
    }

    #[tokio::test]
    async fn test_get_code_non_existing() {
        let store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@test.com".to_string()).unwrap();
        let result = store.get_code(&email).await;
        assert_eq!(result, Err(TwoFACodeStoreError::LoginAttemptIdNotFound));
    }
}