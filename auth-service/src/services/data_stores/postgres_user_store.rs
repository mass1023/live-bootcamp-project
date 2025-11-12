use std::error::Error;

use argon2::{
    password_hash::{SaltString, rand_core::OsRng}, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.0.clone())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let result = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.email.0,
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                Ok(())
            }
            Err(sqlx::Error::Database(db_err)) if db_err.code() == Some("23505".into()) => {
                Err(UserStoreError::UserAlreadyExists)
            }
            Err(_) => Err(UserStoreError::UnexpectedError),
        }
    }

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        let record = sqlx::query!(
            r#"
            SELECT email, password_hash, requires_2fa FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await;

        match record {
            Ok(rec) => {
                let email = Email(rec.email);
                let password = Password(rec.password_hash);
                Ok(Box::leak(Box::new(User {
                    email,
                    password,
                    requires_2fa: rec.requires_2fa
                })))
            }
            Err(sqlx::Error::RowNotFound) => Err(UserStoreError::UserNotFound),
            Err(_) => Err(UserStoreError::UnexpectedError),
        }
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let record = sqlx::query!(
            r#"
            SELECT password_hash FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await;

        let password_hash = match record {
            Ok(rec) => rec.password_hash,
            Err(sqlx::Error::RowNotFound) => return Err(UserStoreError::UserNotFound),
            Err(_) => return Err(UserStoreError::UnexpectedError),
        };

        match verify_password_hash(password_hash, password.to_string()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(UserStoreError::InvalidCredentials),
        }
    }
}

// Helper function to verify if a given password matches an expected hash
#[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error>> {
    let current_span: tracing::Span = tracing::Span::current();
    let handle = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| { 
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;
            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
        })
    });
    let result = match handle.await {
        Ok(res) => res,
        Err(e) => return Err(Box::new(e)),
    };
    if let Err(e) = result {
        return Err(Box::new(e));
    }
    Ok(())
}

// Helper function to hash passwords before persisting them in the database.
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let current_span: tracing::Span = tracing::Span::current(); 
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| { 
            let salt: SaltString = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await;

    result?
}