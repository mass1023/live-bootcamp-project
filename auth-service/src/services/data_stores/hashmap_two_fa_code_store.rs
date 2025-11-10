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
        // Replace any existing code for this email (allows re-login to invalidate old codes)
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
        let new_login_id = LoginAttemptId::default();
        let new_code = TwoFACode::default();
        let result = store.add_code(email.clone(), new_login_id.clone(), new_code.clone()).await;
        // Should succeed and replace the old code
        assert!(result.is_ok());
        // Verify the new code is stored
        let stored = store.get_code(&email).await.unwrap();
        assert_eq!(stored.0, new_login_id);
        assert_eq!(stored.1, new_code);
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