use testresult::TestResult;

use chrono::{DateTime, Days};
use flashcard_gpt_core::dto::deck::{CreateDeckDto, DeckSettings};
use flashcard_gpt_core::dto::deck_card::CreateDeckCardDto;
use flashcard_gpt_core::dto::deck_card_group::CreateDeckCardGroupDto;
use flashcard_gpt_core::dto::history::CreateHistoryDto;
use flashcard_gpt_core::dto::time::Time;
use flashcard_gpt_tests::db::utils::{
    create_card, create_card_group, create_deck, create_deck_repo, create_history_repo, create_tag,
    create_user,
};
use std::sync::Arc;
use tracing::info;

#[tokio::test]
async fn test_create() -> TestResult {
    let repo = create_deck_repo().await?;
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
    let repo = create_deck_repo().await?;
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
    let repo = create_deck_repo().await?;
    let user = create_user("test_relate_card_group").await?;

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

#[tokio::test]
async fn test_get_top_ranked_card_group() -> TestResult {
    let now = DateTime::parse_from_rfc3339("2024-08-01T00:00:00+00:00")?.to_utc();

    let repo = create_deck_repo().await?;
    let user = create_user("test_get_top_ranked_card_group").await?;
    let tag = create_tag().user(&user).name("name").call().await?;

    assert!(repo.get_top_ranked_card_groups(&user, now).await.is_ok());

    let mut decks = vec![];
    let mut deck_cards = vec![];
    let mut deck_card_groups = vec![];

    for deck_index in 0..10 {
        let deck = create_deck()
            .title(format!("sample deck {deck_index}"))
            .settings(DeckSettings {
                daily_limit: deck_index + 1,
            })
            .tags([&tag])
            .user(&user)
            .call()
            .await?;

        let mut cards = vec![];

        for card_id in 0..10 {
            let card = create_card()
                .title(format!("sample card {card_id}"))
                .user(&user)
                .tags([&tag])
                .importance(card_id)
                .difficulty(card_id)
                .call()
                .await?;

            if deck_index % 2 != 0 {
                let deck_card = repo
                    .relate_card(CreateDeckCardDto {
                        deck: deck.id.clone(),
                        card: card.id.clone(),
                    })
                    .await?;
                deck_cards.push(deck_card);
            } else {
                cards.push(card);
            }
        }

        if !cards.is_empty() {
            let card_group = create_card_group()
                .user(&user)
                .title(format!("card group for deck {deck_index}"))
                .importance(deck_index as _)
                .tags([&tag])
                .cards(cards)
                .difficulty(deck_index as _)
                .call()
                .await?;
            let deck_card_group = repo
                .relate_card_group(CreateDeckCardGroupDto {
                    deck: deck.id.clone(),
                    card_group: card_group.id.clone(),
                })
                .await?;

            deck_card_groups.push(deck_card_group);
        }

        decks.push(deck);
    }

    let history = create_history_repo().await?;

    for (index, deck_card) in deck_cards.iter().enumerate() {
        let item = history
            .create_custom(CreateHistoryDto {
                user: user.id.clone(),
                deck_card: deck_card.id.clone().into(),
                deck_card_group: None,
                difficulty: (index % 11) as _,
                time: Some(Time {
                    created_at: now.checked_sub_days(Days::new(index as _)).unwrap(),
                    updated_at: now.checked_sub_days(Days::new(index as _)).unwrap(),
                }),
            })
            .await?;

        info!(?item, "Created history item");
    }

    for (index, deck_card_group) in deck_card_groups.iter().enumerate() {
        let item = history
            .create_custom(CreateHistoryDto {
                user: user.id.clone(),
                deck_card: None,
                deck_card_group: deck_card_group.id.clone().into(),
                difficulty: 0,
                time: Some(Time {
                    created_at: now.checked_sub_days(Days::new(index as _)).unwrap(),
                    updated_at: now.checked_sub_days(Days::new(index as _)).unwrap(),
                }),
            })
            .await?;

        info!(?item, "Created history item");
    }

    let dcg = repo.get_top_ranked_card_groups(&user, now).await;
    assert!(dcg.is_ok(), "{:?}", dcg);

    let dc = repo.get_top_ranked_cards(&user, now).await;
    assert!(dc.is_ok(), "{:?}", dc);

    Ok(())
}
