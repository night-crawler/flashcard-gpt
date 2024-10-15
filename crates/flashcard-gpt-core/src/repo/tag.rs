use crate::model::tag::{CreateTag, Tag};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use itertools::Itertools;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::Span;

pub type TagRepo = GenericRepo<CreateTag, Tag, ()>;

impl TagRepo {
    pub fn new_tag(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "tag", "", "", enable_transactions)
    }

    pub async fn get_or_create_tags(
        &self,
        user_id: impl Into<Thing>,
        tags: impl IntoIterator<Item = Arc<str>>,
    ) -> Result<Vec<Tag>, CoreError> {
        // we assume that slug after slugify stays the same
        let tags = tags
            .into_iter()
            .unique()
            .map(|tag| {
                let slug = slug::slugify(&tag);
                (tag, Arc::from(slug))
            })
            .collect();

        self.get_or_create_tags_raw(user_id, tags).await
    }

    pub async fn get_or_create_tags_raw(
        &self,
        user_id: impl Into<Thing>,
        tags: Vec<(Arc<str>, Arc<str>)>,
    ) -> Result<Vec<Tag>, CoreError> {
        let query = format!(
            r#"
            {begin};
            $slugs = $tag_pairs.map(|$pair| $pair[1]);
            for $pair in $tag_pairs {{
                if select * from tag where slug=$pair[1] && user=$user_id {{
                    continue;
                }};
                insert into tag {{
                    user: $user_id,
                    name: $pair[0],
                    slug: $pair[1]
                }};
            }};
            select * from tag where slug in $slugs && user=$user_id order by slug;
            {commit}
            "#,
            begin = self.begin_transaction_statement(),
            commit = self.commit_transaction_statement()
        );

        let mut response = self
            .db
            .query(query)
            .bind(("user_id", user_id.into()))
            .bind(("tag_pairs", tags))
            .await?;

        response.errors_or_ok()?;

        let tags: Vec<Tag> = response.take(response.num_statements() - 1)?;

        Ok(tags)
    }
}
