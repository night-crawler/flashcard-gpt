use crate::model::user::{RegisterUser, User};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use crate::single_object_query;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub type UserRepo = GenericRepo<RegisterUser, User, ()>;

impl UserRepo {
    pub fn new_user(db: Surreal<Client>, span: tracing::Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "user", "", "", enable_transactions)
    }
    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err)]
    pub async fn list_users(&self) -> Result<Vec<User>, surrealdb::Error> {
        let users: Vec<User> = self.db.select("user").await?;
        Ok(users)
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err, fields(?user)
    )]
    pub async fn create_user(&self, user: RegisterUser) -> Result<User, CoreError> {
        let query = r#"
            CREATE user CONTENT {
                name: $user.name,
                email: $user.email,
                password: crypto::argon2::generate($user.password)
            };
        "#;

        single_object_query!(self.db, query, ("user", user))
    }
}
