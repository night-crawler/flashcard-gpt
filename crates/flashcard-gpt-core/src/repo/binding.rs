use crate::dto::binding::{BindingDto, GetOrCreateBindingDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use crate::{multi_object_query, single_object_query};
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::Span;

pub type BindingRepo = GenericRepo<GetOrCreateBindingDto, BindingDto, ()>;
impl BindingRepo {
    pub fn new_binding(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "binding", "", "user", enable_transactions)
    }

    pub async fn set_banned(&self, pk: impl Into<Thing>) -> Result<BindingDto, CoreError> {
        let query = r#"
        update $pk set time.banned_bot_at = time::now();
        select * from binding where id=$pk fetch user;
        "#;
        single_object_query!(self.db, query, ("pk", pk.into()))
    }

    pub async fn list_all_not_banned(&self) -> Result<Vec<BindingDto>, CoreError> {
        let query = format!(
            r#"
            select * {additional_query} 
            from {table_name}
            where time.banned_bot_at = none
            {fetch}
            "#,
            table_name = self.table_name,
            fetch = self.fetch_statement(),
            additional_query = self.additional_query
        );

        multi_object_query!(self.db, &query,)
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
            $binding = select * from binding where source_id=$dto.source_id fetch user;
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

        single_object_query!(self.db, query, ("dto", dto))
    }
}
