use flashcard_gpt_core::dto::tag::CreateTagDto;
use flashcard_gpt_core::repo::tag::TagRepo;
use flashcard_gpt_tests::db::utils::{create_tag_repo, create_user};
use flashcard_gpt_tests::db::TestDbExt;
use flashcard_gpt_tests::db::TEST_DB;
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
    let repo = create_tag_repo().await?;
    let user = create_user("tag_create_22").await?;

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
        .get_or_create_tags_raw(user.id.clone(), tags.clone())
        .await?;
    assert_eq!(created_tags.len(), 2, "{created_tags:#?}");

    let created_tags = repo.get_or_create_tags_raw(user.id.clone(), tags).await?;
    assert_eq!(created_tags.len(), 2, "{created_tags:#?}");

    Ok(())
}

#[tokio::test]
async fn test_duplicates() {
    let user = create_user("test_duplicates").await.unwrap();
    let unique_tag_pairs = vec![
        (Arc::from("q"), Arc::from("q")),
        (Arc::from("w"), Arc::from("w")),
        (Arc::from("f"), Arc::from("f")),
        (Arc::from("er"), Arc::from("er")),
        (Arc::from("a"), Arc::from("a")),
        (Arc::from("sad"), Arc::from("sad")),
    ];

    let repo = TagRepo::new_tag(
        TEST_DB.get_client().await.unwrap(),
        span!(Level::INFO, "tag_create"),
        true,
    );
    let created_tags = repo
        .get_or_create_tags_raw(user.id.clone(), unique_tag_pairs.clone())
        .await
        .unwrap();
    assert_eq!(
        created_tags.len(),
        unique_tag_pairs.len(),
        "{created_tags:#?}"
    );

    let next_tags = vec![
        (Arc::from("sad"), Arc::from("sad")),
        (Arc::from("wew"), Arc::from("wew")),
    ];

    let created_tags = repo
        .get_or_create_tags_raw(user.id.clone(), next_tags.clone())
        .await
        .unwrap();
    assert_eq!(created_tags.len(), next_tags.len(), "{created_tags:#?}");
    assert!(created_tags
        .into_iter()
        .all(|t| ["sad", "wew"].contains(&t.slug.as_ref())));
}
