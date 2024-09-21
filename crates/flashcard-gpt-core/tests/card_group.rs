use flashcard_gpt_core::dto::card_group::CreateCardGroupDto;
use flashcard_gpt_core::repo::card_group::CardGroupRepo;
use flashcard_gpt_tests::db::utils::{create_card, create_tag, create_user};
use flashcard_gpt_tests::db::TestDbExt;
use flashcard_gpt_tests::db::TEST_DB;
use serde_json::json;
use std::sync::Arc;
use testresult::TestResult;
use tracing::{span, Level};

#[tokio::test]
async fn test_create() -> TestResult {
    let db = TEST_DB.get_client().await?;
    let repo = CardGroupRepo::new_card_group(db, span!(Level::INFO, "card_group_create"), true);
    let user = create_user("card_group_create").await?;
    let tag = create_tag()
        .user(&user)
        .name("card_group_create")
        .slug("card_group_create")
        .call()
        .await?;

    let card1 = create_card()
        .user(&user)
        .title("card1")
        .tags([&tag])
        .call()
        .await?;
    let card2 = create_card()
        .user(&user)
        .title("card2")
        .tags([&tag])
        .call()
        .await?;

    let card_group = CreateCardGroupDto {
        user: user.id,
        title: Arc::from("title"),
        importance: 1,
        tags: vec![tag.id],
        cards: vec![card1.id, card2.id],
        difficulty: 2,
        data: Some(Arc::from(json!({
            "a": "b"
        }))),
    };

    let card_group = repo.create(card_group).await?;
    assert_eq!(card_group.title.as_ref(), "title");
    assert_eq!(card_group.cards.len(), 2);
    assert_eq!(card_group.tags.len(), 1);

    Ok(())
}
