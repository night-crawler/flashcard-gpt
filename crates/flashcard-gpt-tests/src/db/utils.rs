use crate::db::{TestDbExt, TEST_DB};
use bon::builder;
use flashcard_gpt_core::dto::card::{CardDto, CreateCardDto};
use flashcard_gpt_core::dto::card_group::{CardGroupDto, CreateCardGroupDto};
use flashcard_gpt_core::dto::deck::{CreateDeckDto, DeckDto, DeckSettings};
use flashcard_gpt_core::dto::tag::{CreateTagDto, TagDto};
use flashcard_gpt_core::dto::user::{RegisterUserDto, User};
use flashcard_gpt_core::repo::card::CardRepo;
use flashcard_gpt_core::repo::card_group::CardGroupRepo;
use flashcard_gpt_core::repo::deck::DeckRepo;
use flashcard_gpt_core::repo::tag::TagRepo;
use flashcard_gpt_core::repo::user::UserRepo;
use std::sync::Arc;
use surrealdb::sql::Thing;
use testresult::TestResult;
use tracing::{span, Level};

pub async fn create_user_repo() -> TestResult<UserRepo> {
    let db = TEST_DB.get_client().await?;
    Ok(UserRepo::new_user(
        db,
        span!(Level::INFO, "user_repo"),
        true,
    ))
}

pub async fn create_tag_repo() -> TestResult<TagRepo> {
    let db = TEST_DB.get_client().await?;
    Ok(TagRepo::new_tag(db, span!(Level::INFO, "tag_repo"), true))
}

pub async fn create_deck_repo() -> TestResult<DeckRepo> {
    let db = TEST_DB.get_client().await?;
    Ok(DeckRepo::new_deck(
        db,
        span!(Level::INFO, "deck_repo"),
        true,
    ))
}

pub async fn create_card_repo() -> TestResult<CardRepo> {
    let db = TEST_DB.get_client().await?;
    Ok(CardRepo::new_card(
        db,
        span!(Level::INFO, "card_repo"),
        true,
    ))
}

pub async fn create_card_group_repo() -> TestResult<CardGroupRepo> {
    let db = TEST_DB.get_client().await?;
    Ok(CardGroupRepo::new_card_group(
        db,
        span!(Level::INFO, "card_group_repo"),
        true,
    ))
}

pub async fn create_user(name: &str) -> TestResult<User> {
    let repo = create_user_repo().await?;

    let user = repo
        .create_user(RegisterUserDto {
            email: format!("{}@example.com", name.to_lowercase()).into(),
            name: name.to_string().into(),
            password: name.to_string().into(),
        })
        .await?;

    Ok(user)
}

#[builder]
pub async fn create_tag<U>(user: U, name: &str, slug: Option<&str>) -> TestResult<TagDto>
where
    U: Into<Thing>,
{
    let repo = create_tag_repo().await?;

    let tag = repo
        .create(CreateTagDto {
            name: Arc::from(name),
            slug: Arc::from(slug.unwrap_or(name)),
            user: user.into(),
        })
        .await?;

    Ok(tag)
}

#[builder]
pub async fn create_deck<U, Title, Tag, TagIter>(
    user: U,
    title: Title,
    parent: Option<Thing>,
    settings: Option<DeckSettings>,
    tags: Option<TagIter>,
) -> TestResult<DeckDto>
where
    TagIter: IntoIterator<Item = Tag>,
    Tag: Into<Thing>,
    U: Into<Thing>,
    Title: Into<Arc<str>>,
{
    let repo = create_deck_repo().await?;

    let tags = if let Some(tags) = tags {
        tags.into_iter().map(|t| t.into()).collect()
    } else {
        vec![]
    };

    let title = title.into();

    let deck = repo
        .create(CreateDeckDto {
            description: Some(Arc::from(format!("description for {}", title.clone()))),
            parent: parent.map(Into::into),
            settings,
            user: user.into(),
            title,
            tags,
        })
        .await?;
    Ok(deck)
}

#[builder]
pub async fn create_card<U, Tag, TagIter, Title>(
    user: U,
    title: Title,
    front: Option<&str>,
    back: Option<&str>,
    hints: Option<Vec<&str>>,
    difficulty: Option<u8>,
    importance: Option<u8>,
    tags: Option<TagIter>,
) -> TestResult<CardDto>
where
    TagIter: IntoIterator<Item = Tag>,
    Tag: Into<Thing>,
    U: Into<Thing>,
    Title: Into<Arc<str>>,
{
    let repo = create_card_repo().await?;

    let tags = tags
        .into_iter()
        .flat_map(|t| t.into_iter())
        .map(|t| t.into())
        .collect::<Vec<_>>();
    let hints = hints
        .into_iter()
        .flat_map(|hints| hints.into_iter())
        .map(|hint| hint.into())
        .collect::<Vec<_>>();

    let title = title.into();

    let card = repo
        .create(CreateCardDto {
            user: user.into(),
            title: title.clone(),
            front: front.map(Arc::from).or(Some(title.clone())),
            back: back.map(Arc::from).or(Some(title.clone())),
            hints,
            difficulty: difficulty.unwrap_or(0),
            importance: importance.unwrap_or(0),
            data: None,
            tags,
        })
        .await?;

    Ok(card)
}

#[builder]
pub async fn create_card_group<U, Tag, Card, TagIter, CardIter, Title>(
    user: U,
    title: Title,
    cards: CardIter,
    difficulty: Option<u8>,
    importance: Option<u8>,
    tags: Option<TagIter>,
) -> TestResult<CardGroupDto>
where
    CardIter: IntoIterator<Item = Card>,
    TagIter: IntoIterator<Item = Tag>,
    Tag: Into<Thing>,
    Card: Into<Thing>,
    U: Into<Thing>,
    Title: Into<Arc<str>>,
{
    let repo = create_card_group_repo().await?;

    let tags = tags
        .into_iter()
        .flat_map(|t| t.into_iter())
        .map(|t| t.into())
        .collect::<Vec<_>>();

    let cards = cards.into_iter().map(|c| c.into()).collect::<Vec<_>>();

    let card_group = repo
        .create(CreateCardGroupDto {
            user: user.into(),
            title: title.into(),
            importance: importance.unwrap_or(0),
            tags,
            cards,
            difficulty: difficulty.unwrap_or(0),
            data: None,
        })
        .await?;

    Ok(card_group)
}
