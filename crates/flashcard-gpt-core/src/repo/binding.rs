use crate::dto::binding::{BindingDto, GetOrCreateBindingDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type BindingRepo = GenericRepo<GetOrCreateBindingDto, BindingDto, ()>;
impl BindingRepo {
    pub fn new_binding(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "binding", "user", enable_transactions)
    }

    #[tracing::instrument(level = "debug", skip_all, parent = self.span.clone(), err, fields(source_id)
    )]
    pub async fn get_by_source_id(
        &self,
        source_id: Arc<str>,
    ) -> Result<Option<BindingDto>, CoreError> {
        let query = r#"
            select * from binding where source_id=$source_id fetch user;
        "#;

        let mut response = self.db.query(query).bind(("source_id", source_id)).await?;
        response.errors_or_ok()?;

        Ok(response.take(0)?)
    }

    #[tracing::instrument(level = "info", skip_all, parent = self.span.clone(), err, fields(?dto))]
    pub async fn get_or_create_binding(
        &self,
        dto: GetOrCreateBindingDto,
    ) -> Result<BindingDto, CoreError> {
        let query = r#"
            begin transaction;
            $binding = select * from binding where source_id=$source_id fetch user;
            if $binding {
                return $binding[0];
            };

            $user_id = (select id from user where email=$dto.email)[0].id ?:
                (create user content {
                    email: $dto.email,
                    name: $dto.name,
                    password: crypto::argon2::generate($dto.password)
                } return id)[0].id;

            $id = (create binding content {
                source_id: $dto.source_id,
                type_name: $dto.type_name,
                user: $user_id,
                data: $dto.data
            } return id)[0].id;
            return select * from binding where id=$id fetch user;
            commit transaction;
        "#;

        let source_id = dto.source_id.clone();

        let mut response = self
            .db
            .query(query)
            .bind(("source_id", source_id.clone()))
            .bind(("dto", dto))
            .await?;
        response.errors_or_ok()?;

        let binding: Option<BindingDto> = response.take(0)?;
        let binding = binding.ok_or(CoreError::CreateError(source_id))?;

        Ok(binding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::tests::TEST_DB;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_or_create_binding() -> Result<(), CoreError> {
        let db = TEST_DB.get_client().await?;
        let repo = BindingRepo::new_binding(db, tracing::span!(tracing::Level::INFO, "test"), true);

        let result = repo.get_by_source_id("source_id".into()).await?;
        assert!(result.is_none());

        let dto = GetOrCreateBindingDto {
            source_id: "source_id".into(),
            type_name: "sample".into(),
            email: "qemail@email.com".into(),
            name: "name".into(),
            password: "password".into(),
            data: json!({
                "a": "b"
            })
            .into(),
        };
        let binding = repo.get_or_create_binding(dto).await?;
        assert_eq!(binding.source_id.as_ref(), "source_id");

        Ok(())
    }
}
