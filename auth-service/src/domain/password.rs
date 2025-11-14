use color_eyre::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Password(pub String);

impl Password {
    pub fn parse(s: String) -> Result<Self, String> {
        if s.len() >= 8 {
            Ok(Password(s))
        } else {
            Err("Password must be at least 8 characters long".to_string())
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_password() {
        let password_str = "short";
        let password = Password::parse(password_str.to_string());
        assert!(password.is_err()); 
    }

    #[test]
    fn test_valid_password() {
        let password_str = "longenough";
        let password = Password::parse(password_str.to_string());
        assert!(password.is_ok());
        assert_eq!(password.unwrap().as_ref(), password_str);   
    }
} 