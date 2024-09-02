use crate::dto::user::User;
use crate::error::CoreError;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub struct UserRepo {
    db: Surreal<Client>,
}

impl UserRepo {
    pub async fn new(db: Surreal<Client>) -> Self {
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
        let created_user = created_user.ok_or(CoreError::CreateUserError(email))?;
        Ok(created_user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::user::User;
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Root;

    #[tokio::test]
    async fn test_create_surrealdb_connection() -> Result<(), CoreError> {
        let db = Surreal::new::<Ws>("127.0.0.1:8477").await?;

        // Signin as a namespace, database, or root user
        db.signin(Root {
            username: "root",
            password: "root",
        })
            .await?;

        // Select a specific namespace / database
        db.use_ns("flashcards_gpt").use_db("flashcards").await?;

        let repo = UserRepo::new(db).await;
        let users = repo.list_users().await?;
        println!("{:?}", users);

        repo.create_user(User {
            id: None,
            email: "bla@bla.com".to_string().into(),
            name: "Bla".to_string().into(),
            password: "sample".to_string().into(),
            time: None,
        }).await?;


        Ok(())
    }
}