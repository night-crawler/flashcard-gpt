use flashcard_gpt_core::model::card_group::{CreateCardGroup, UpdateCardGroup};
use flashcard_gpt_tests::db::utils::{
    create_card, create_card_group_repo, create_tag, create_user,
};
use serde_json::json;
use std::sync::Arc;
use testresult::TestResult;

#[tokio::test]
async fn test_create() -> TestResult {
    let repo = create_card_group_repo().await?;
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

    let card_group = CreateCardGroup {
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

#[tokio::test]
async fn test_patch() -> TestResult {
    let repo = create_card_group_repo().await?;
    let user = create_user("card_group_patch").await?;
    let tag = create_tag()
        .user(&user)
        .name("card_group_patch")
        .slug("card_group_patch")
        .call()
        .await?;

    let card_group = CreateCardGroup {
        user: user.id,
        title: Arc::from("title"),
        importance: 1,
        tags: vec![tag.id],
        cards: vec![],
        difficulty: 2,
        data: Some(Arc::from(json!({
            "a": "b"
        }))),
    };
    let card_group = repo.create(card_group).await?;

    let cg = repo
        .patch(
            card_group.id.clone(),
            UpdateCardGroup {
                importance: Some(3),
                difficulty: Some(4),
            },
        )
        .await?;
    assert_eq!(cg.importance, 3);
    assert_eq!(cg.difficulty, 4);

    let cg = repo
        .patch(
            card_group.id.clone(),
            UpdateCardGroup {
                importance: None,
                difficulty: None,
            },
        )
        .await?;
    assert_eq!(cg.importance, 3);
    assert_eq!(cg.difficulty, 4);

    let cg = repo
        .patch(
            card_group.id.clone(),
            UpdateCardGroup {
                importance: Some(7),
                difficulty: None,
            },
        )
        .await?;
    assert_eq!(cg.importance, 7);
    assert_eq!(cg.difficulty, 4);

    Ok(())
}
