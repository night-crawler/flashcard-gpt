use crate::dto::user::{RegisterUserDto, User};
use crate::error::CoreError;
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub type UserRepo = GenericRepo<RegisterUserDto, User, ()>;

impl UserRepo {
    pub fn new_user(db: Surreal<Client>, span: tracing::Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "user", "", enable_transactions)
    }
    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err)]
    pub async fn list_users(&self) -> Result<Vec<User>, surrealdb::Error> {
        let users: Vec<User> = self.db.select("user").await?;
        Ok(users)
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err, fields(?user)
    )]
    pub async fn create_user(&self, user: RegisterUserDto) -> Result<User, CoreError> {
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
}
