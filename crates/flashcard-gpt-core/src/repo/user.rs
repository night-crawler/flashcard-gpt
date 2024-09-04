use crate::dto::user::User;
use crate::error::CoreError;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use crate::ext::record_id::RecordIdExt;

#[derive(Debug, Clone)]
pub struct UserRepo {
    db: Surreal<Client>,
}

impl UserRepo {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, surrealdb::Error> {
        let users: Vec<User> = self.db.select("user").await?;
        Ok(users)
    }

    pub async fn create_user(&self, user: User) -> Result<User, CoreError> {
        let query = r#"
            CREATE user CONTENT {
                name: $user.name,
                email: $user.email,
                password: crypto::argon2::generate($user.password)
            };
        "#;

        let email = user.email.clone();

        let mut response = self.db.query(query)
            .bind(("user", user))
            .await?;

        let created_user: Option<User> = response.take(0)?;
        let created_user = created_user.ok_or(CoreError::CreateError(email))?;
        Ok(created_user)
    }

    pub async fn get_user_by_id(&self, id: impl RecordIdExt) -> Result<User, CoreError> {
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
    use crate::tests::TEST_DB;
    use crate::tests::utils::create_user;

    #[tokio::test]
    async fn test_create_user() -> Result<(), CoreError> {
        let db = TEST_DB.get_client().await?;
        let repo = UserRepo::new(db);

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