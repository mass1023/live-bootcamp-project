use regex::Regex;

use color_eyre::Result;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Email(pub String);

impl Email {
    pub fn parse(s: String) -> Result<Self, String> {
        if s.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if email_regex.is_match(s.as_str()) {
            Ok(Email(s))
        } 
        else {
            Err(format!("Invalid email address: {}", s))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_email() {
        let email_str = "";
        let email = Email::parse(email_str.to_string());
        assert!(email.is_err());

        let email_str = "invalid-email";
        let email = Email::parse(email_str.to_string());
        assert!(email.is_err());
    }

    #[test]
    fn test_valid_email() {
        let email_str = "test@test.com";
        let email = Email::parse(email_str.to_string());
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_ref(), email_str); 
    }
}