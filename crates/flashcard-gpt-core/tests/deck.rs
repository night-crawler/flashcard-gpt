use testresult::TestResult;

use flashcard_gpt_core::dto::deck::{CreateDeckDto, DeckSettings};
use flashcard_gpt_core::dto::deck_card::CreateDeckCardDto;
use flashcard_gpt_core::dto::deck_card_group::CreateDeckCardGroupDto;
use flashcard_gpt_core::repo::deck::DeckRepo;
use flashcard_gpt_tests::db::utils::{
    create_card, create_card_group, create_deck, create_tag, create_user,
};
use flashcard_gpt_tests::db::TestDbExt;
use flashcard_gpt_tests::db::TEST_DB;
use std::sync::Arc;
use tracing::{span, Level};

#[tokio::test]
async fn test_create() -> TestResult {
    let db = TEST_DB.get_client().await?;
    let repo = DeckRepo::new_deck(db.clone(), span!(Level::INFO, "deck_create"), false);
    let user = create_user("deck_create").await?;

    let tag = create_tag().user(&user).name("name").call().await?;
    let deck = repo
        .create(CreateDeckDto {
            description: Some(Arc::from("description")),
            parent: None,
            user: user.id.clone(),
            title: Arc::from("title"),
            tags: vec![tag.id.clone()],
            settings: None,
        })
        .await?;

    let deck = repo.get_by_id(deck.id.clone()).await?;

    assert_eq!(deck.description.as_deref(), Some("description"));
    assert!(deck.parent.is_none());

    let deck2 = create_deck()
        .title("sample deck 2")
        .user(&user)
        .tags([&tag])
        .parent(deck.id.clone())
        .settings(DeckSettings { daily_limit: 200 })
        .call()
        .await?;

    assert_eq!(deck2.parent.as_ref().unwrap(), &deck.id);

    Ok(())
}

#[tokio::test]
async fn test_relate_card() -> TestResult {
    let db = TEST_DB.get_client().await?;
    let repo = DeckRepo::new_deck(
        db.clone(),
        span!(Level::INFO, "deck_create_relation"),
        false,
    );
    let user = create_user("deck_create_relation_card").await?;

    let tag = create_tag().user(&user).name("name").call().await?;

    let deck1 = create_deck()
        .title("sample deck")
        .user(&user)
        .tags([&tag])
        .call()
        .await?;

    let deck2 = create_deck()
        .title("sample deck 2")
        .user(&user)
        .tags([&tag])
        .call()
        .await?;

    let mut relations = vec![];

    for _ in 0..10 {
        let card = create_card()
            .user(&user)
            .tags([&tag])
            .title(format!("card {}", 1))
            .call()
            .await?;

        let relation = repo
            .relate_card(CreateDeckCardDto {
                deck: deck1.id.clone(),
                card: card.id.clone(),
            })
            .await?;

        relations.push(relation);
    }

    let cards = repo.list_cards(&user, &deck1).await?;
    assert_eq!(cards.len(), 10);

    let cards2 = repo.list_cards(&user, &deck2).await?;
    assert!(cards2.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_relate_card_group() -> TestResult {
    let db = TEST_DB.get_client().await?;
    let repo = DeckRepo::new_deck(
        db.clone(),
        span!(Level::INFO, "deck_create_relation"),
        false,
    );
    let user = create_user("deck_create_relation_card_group").await?;

    let tag = create_tag().user(&user).name("name").call().await?;

    let deck1 = create_deck()
        .title("sample deck")
        .user(&user)
        .tags([&tag])
        .call()
        .await?;

    let mut cards = vec![];
    for _ in 0..10 {
        let card = create_card()
            .user(&user)
            .tags([&tag])
            .title(format!("card {}", 1))
            .call()
            .await?;
        cards.push(card);
    }

    let card_group = create_card_group()
        .user(&user)
        .title("card group")
        .importance(1)
        .tags([&tag])
        .cards(cards)
        .difficulty(2)
        .call()
        .await?;

    let relation = repo
        .relate_card_group(CreateDeckCardGroupDto {
            deck: deck1.id.clone(),
            card_group: card_group.id.clone(),
        })
        .await?;

    assert_eq!(relation.card_group.cards.len(), 10);

    Ok(())
}
