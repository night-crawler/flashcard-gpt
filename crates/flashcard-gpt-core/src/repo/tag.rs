use crate::dto::tag::{CreateTagDto, TagDto};
use crate::error::CoreError;
use crate::ext::response_ext::ResponseExt;
use crate::repo::generic_repo::GenericRepo;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::Span;

pub type TagRepo = GenericRepo<CreateTagDto, TagDto, ()>;

impl TagRepo {
    pub fn new_tag(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "tag", "", enable_transactions)
    }

    pub async fn get_or_create_tags(
        &self,
        user_id: Thing,
        tags: Vec<(Arc<str>, Arc<str>)>,
    ) -> Result<Vec<TagDto>, CoreError> {
        let query = format!(
            r#"
            {begin};
            $slugs = $tag_pairs.map(|$pair| $pair[1]);
            for $pair in $tag_pairs {{
                insert into tag {{
                    user: $user_id,
                    name: $pair[0],
                    slug: $pair[1]
                }};
            }};
            select * from tag where slug in $slugs order by slug;
            {commit}
            "#,
            begin = self.begin_transaction_statement(),
            commit = self.commit_transaction_statement()
        );

        let mut response = self
            .db
            .query(query)
            .bind(("user_id", user_id))
            .bind(("tag_pairs", tags))
            .await?;

        response.errors_or_ok()?;

        let tags: Vec<TagDto> = response.take(response.num_statements() - 1)?;

        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_user;
    use crate::tests::TEST_DB;
    use std::sync::Arc;
    use testresult::TestResult;
    use tracing::{span, Level};

    #[tokio::test]
    async fn test_create() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = TagRepo::new_tag(db, span!(Level::INFO, "tag_create"), true);
        let user = create_user("tag_create").await?;

        let tag = CreateTagDto {
            user: user.id.clone(),
            name: Arc::from("title"),
            slug: Arc::from("slug"),
        };

        let tag = repo.create(tag).await?;
        assert_eq!(tag.name.as_ref(), "title");
        assert_eq!(tag.slug.as_ref(), "slug");

        for i in 0..10 {
            let tag = CreateTagDto {
                user: user.id.clone(),
                name: Arc::from(format!("title {i}")),
                slug: Arc::from(format!("slug-{i}")),
            };

            let _ = repo.create(tag).await?;
        }

        let tags = repo.list_by_user_id(user.id).await?;
        assert_eq!(tags.len(), 11);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_tags() -> TestResult {
        let db = TEST_DB.get_client().await?;
        let repo = TagRepo::new_tag(db, span!(Level::INFO, "tag_create"), true);
        let user = create_user("tag_create").await?;

        repo.create(CreateTagDto {
            user: user.id.clone(),
            name: Arc::from("not a title"),
            slug: Arc::from("not-a-title"),
        })
        .await?;

        let tags = vec![
            (Arc::from("title"), Arc::from("slug")),
            (Arc::from("title2"), Arc::from("slug2")),
        ];

        let created_tags = repo
            .get_or_create_tags(user.id.clone(), tags.clone())
            .await?;
        assert_eq!(created_tags.len(), 2);

        let created_tags = repo.get_or_create_tags(user.id.clone(), tags).await?;
        assert_eq!(created_tags.len(), 2);

        Ok(())
    }
}
