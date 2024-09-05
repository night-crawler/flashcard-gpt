use crate::dto::user::User;
use crate::error::CoreError;
use crate::ext::record_id::RecordIdExt;
use std::fmt::Debug;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct UserRepo {
    db: Surreal<Client>,
    span: tracing::Span,
}

impl UserRepo {
    pub fn new(db: Surreal<Client>, span: tracing::Span) -> Self {
        Self { db, span }
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err)]
    pub async fn list_users(&self) -> Result<Vec<User>, surrealdb::Error> {
        let users: Vec<User> = self.db.select("user").await?;
        Ok(users)
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err, fields(?user))]
    pub async fn create_user(&self, user: User) -> Result<User, CoreError> {
        let query = r#"
            CREATE user CONTENT {
                name: $user.name,
                email: $user.email,
                password: crypto::argon2::generate($user.password)
            };
        "#;

        let email = user.email.clone();

        let mut response = self.db.query(query).bind(("user", user)).await?;

        let created_user: Option<User> = response.take(0)?;
        let created_user = created_user.ok_or(CoreError::CreateError(email))?;
        Ok(created_user)
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err, fields(id))]
    pub async fn get_user_by_id(&self, id: impl RecordIdExt + Debug) -> Result<User, CoreError> {
        let id = id.record_id();
        let user: Option<User> = self.db.select(id.clone()).await?;
        if let Some(user) = user {
            Ok(user)
        } else {
            Err(CoreError::NotFound("user", id.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create_user() -> Result<(), CoreError> {
        let db = TEST_DB.get_client().await?;
        let repo = UserRepo::new(db, span!(Level::INFO, "user_create"));

        let users = repo.list_users().await?;
        assert!(users.is_empty());

        let user = create_user("Bla").await?;

        assert_eq!(user.email.as_ref(), "bla@example.com");
        assert_eq!(user.name.as_ref(), "Bla");

        assert!(!user.password.is_empty());
        assert!(user.time.is_some());

        let user = repo.get_user_by_id(user.id.unwrap()).await?;
        println!("{:?}", user);

        Ok(())
    }
}
