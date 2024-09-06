use crate::dto::tag::{CreateTagDto, TagDto};
use crate::repo::generic_repo::GenericRepo;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::Span;

pub type TagRepo = GenericRepo<CreateTagDto, TagDto, ()>;

impl TagRepo {
    pub fn new_tag(db: Surreal<Client>, span: Span, enable_transactions: bool) -> Self {
        Self::new(db, span, "tag", "", enable_transactions)
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
            user: user.id,
            name: Arc::from("title"),
            slug: Arc::from("slug"),
        };

        let tag = repo.create(tag).await?;
        assert_eq!(tag.name.as_ref(), "title");
        assert_eq!(tag.slug.as_ref(), "slug");

        Ok(())
    }
}
