use chrono::Duration;
use flashcard_gpt_core::dto::deck_card::CreateDeckCardDto;
use flashcard_gpt_core::dto::deck_card_group::CreateDeckCardGroupDto;
use flashcard_gpt_core::dto::history::CreateHistoryDto;
use flashcard_gpt_core::dto::time::Time;
use flashcard_gpt_tests::db::utils::{
    create_card, create_card_group, create_deck, create_deck_repo, create_history_repo, create_tag,
    create_user,
};
use testresult::TestResult;

#[tokio::test]
async fn test_create() -> TestResult {
    let time = chrono::DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z")?;

    let deck_repo = create_deck_repo().await?;
    let repo = create_history_repo().await?;
    let user = create_user("history_create").await?;

    let tag = create_tag()
        .name("tag1")
        .slug("tag1")
        .user(user.id.clone())
        .call()
        .await?;

    let deck = create_deck()
        .tags([&tag])
        .title("deck1")
        .user(user.id.clone())
        .call()
        .await?;

    let card = create_card()
        .user(user.id.clone())
        .title("card1")
        .front("front1")
        .back("back1")
        .tags([&tag])
        .call()
        .await?;

    let deck_card = deck_repo
        .relate_card(CreateDeckCardDto {
            deck: deck.id.clone(),
            card: card.id.clone(),
        })
        .await?;

    let history = repo
        .create_custom(CreateHistoryDto {
            user: user.id.clone(),
            deck_card: Some(deck_card.id.clone()),
            deck_card_group: None,
            difficulty: 3,
            time: None,
            hide_for: None,
        })
        .await?;
    assert_ne!(history.time.created_at, time);
    assert_ne!(history.time.updated_at, time);

    let history = repo
        .create_custom(CreateHistoryDto {
            user: user.id.clone(),
            deck_card: Some(deck_card.id.clone()),
            deck_card_group: None,
            difficulty: 3,
            time: Some(Time {
                created_at: time.to_utc(),
                updated_at: time.to_utc(),
                deleted_at: None,
            }),
            hide_for: Some(Duration::seconds(10000)),
        })
        .await?;

    assert_eq!(history.difficulty, 3);
    assert!(history.deck_card.is_some());
    assert!(history.deck_card_group.is_none());
    assert_eq!(history.time.created_at, time.to_utc());
    assert_eq!(history.time.updated_at, time.to_utc());

    let card_group = create_card_group()
        .user(user.id.clone())
        .title("card_group1")
        .tags([&tag])
        .cards([&card])
        .call()
        .await?;

    let deck_card_group = deck_repo
        .relate_card_group(CreateDeckCardGroupDto {
            deck: deck.id.clone(),
            card_group: card_group.id.clone(),
        })
        .await?;

    let history = repo
        .create_custom(CreateHistoryDto {
            user: user.id.clone(),
            deck_card: None,
            deck_card_group: Some(deck_card_group.id.clone()),
            difficulty: 2,
            time: None,
            hide_for: Some(Duration::seconds(10000)),
        })
        .await?;

    assert!(history.deck_card.is_none());
    assert!(history.deck_card_group.is_some());

    Ok(())
}
