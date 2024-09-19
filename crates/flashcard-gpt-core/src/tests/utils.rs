use crate::dto::card::{CardDto, CreateCardDto};
use crate::dto::card_group::{CardGroupDto, CreateCardGroupDto};
use crate::dto::deck::{CreateDeckDto, DeckDto, DeckSettings};
use crate::dto::tag::{CreateTagDto, TagDto};
use crate::dto::user::{RegisterUserDto, User};
use crate::error::CoreError;
use crate::ext::mutex::MutexExt;
use crate::repo::card::CardRepo;
use crate::repo::card_group::CardGroupRepo;
use crate::repo::deck::DeckRepo;
use crate::repo::tag::TagRepo;
use crate::repo::user::UserRepo;
use crate::tests::surreal_test_container::{SurrealDbTestContainer, SURREALDB_PORT};
use crate::tests::TEST_DB;
use bon::builder;
use log::info;
use std::sync::{Arc, Mutex};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use tracing::{span, Level};

#[derive(Default)]
pub struct TestDb {
    container: Mutex<Option<ContainerAsync<SurrealDbTestContainer>>>,
    client: Mutex<Option<Surreal<Client>>>,
}

impl TestDb {
    pub const fn new() -> Self {
        Self {
            container: Mutex::new(None),
            client: Mutex::new(None),
        }
    }

    pub async fn get_client(&self) -> Result<Surreal<Client>, CoreError> {
        let mut client = self.client.lock_sync()?;
        if let Some(client) = client.as_mut() {
            return Ok(client.clone());
        }

        let (container, db) = prepare_database().await?;
        *client = Some(db);
        *self.container.lock_sync()? = Some(container);
        Ok(client.clone().unwrap())
    }
}

pub async fn prepare_database(
) -> Result<(ContainerAsync<SurrealDbTestContainer>, Surreal<Client>), CoreError> {
    let _ = pretty_env_logger::try_init();
    let node = SurrealDbTestContainer::default().start().await?;
    let host_port = node.get_host_port_ipv4(SURREALDB_PORT).await?;
    let url = format!("127.0.0.1:{host_port}");

    let db: Surreal<Client> = Surreal::init();
    db.connect::<Ws>(url).await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("test").use_db("test").await?;

    let migration_data =
        include_str!("../../db-migrations/migrations/20240902_185441_Initial.surql");
    let mut response = db.query(migration_data).await?;

    let mut last_error = None;

    for (id, err) in response.take_errors() {
        log::error!("{id}: {err}");
        last_error = Some(err);
    }

    if let Some(err) = last_error {
        Err(err)?;
    }

    info!("Migration complete");

    Ok((node, db))
}

pub async fn create_user(name: &str) -> Result<User, CoreError> {
    let db = TEST_DB.get_client().await?;
    let repo = UserRepo::new_user(db, span!(Level::INFO, "user_create"), true);

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
pub async fn create_tag<U>(user: U, name: &str, slug: Option<&str>) -> Result<TagDto, CoreError>
where
    U: Into<Thing>,
{
    let db = TEST_DB.get_client().await?;
    let repo = TagRepo::new_tag(db, span!(Level::INFO, "tag_create"), false);

    repo.create(CreateTagDto {
        name: Arc::from(name),
        slug: Arc::from(slug.unwrap_or(name)),
        user: user.into(),
    })
    .await
}

#[builder]
pub async fn create_deck<U, Title, Tag, TagIter>(
    user: U,
    title: Title,
    parent: Option<Thing>,
    settings: Option<DeckSettings>,
    tags: Option<TagIter>,
) -> Result<DeckDto, CoreError>
where
    TagIter: IntoIterator<Item = Tag>,
    Tag: Into<Thing>,
    U: Into<Thing>,
    Title: Into<Arc<str>>,
{
    let db = TEST_DB.get_client().await?;
    let repo = DeckRepo::new_deck(db, span!(Level::INFO, "deck_create"), false);

    let tags = if let Some(tags) = tags {
        tags.into_iter().map(|t| t.into()).collect()
    } else {
        vec![]
    };

    let title = title.into();

    repo.create(CreateDeckDto {
        description: Some(Arc::from(format!("description for {}", title.clone()))),
        parent: parent.map(Into::into),
        settings,
        user: user.into(),
        title,
        tags,
    })
    .await
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
) -> Result<CardDto, CoreError>
where
    TagIter: IntoIterator<Item = Tag>,
    Tag: Into<Thing>,
    U: Into<Thing>,
    Title: Into<Arc<str>>,
{
    let db = TEST_DB.get_client().await?;
    let repo = CardRepo::new_card(db, span!(Level::INFO, "card_create"), false);

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

    repo.create(CreateCardDto {
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
    .await
}

#[builder]
pub async fn create_card_group<U, Tag, Card, TagIter, CardIter, Title>(
    user: U,
    title: Title,
    cards: CardIter,
    difficulty: Option<u8>,
    importance: Option<u8>,
    tags: Option<TagIter>,
) -> Result<CardGroupDto, CoreError>
where
    CardIter: IntoIterator<Item = Card>,
    TagIter: IntoIterator<Item = Tag>,
    Tag: Into<Thing>,
    Card: Into<Thing>,
    U: Into<Thing>,
    Title: Into<Arc<str>>,
{
    let db = TEST_DB.get_client().await?;
    let repo =
        CardGroupRepo::new_card_group(db.clone(), span!(Level::INFO, "card_group_create"), false);

    let tags = tags
        .into_iter()
        .flat_map(|t| t.into_iter())
        .map(|t| t.into())
        .collect::<Vec<_>>();
    
    let cards = cards.into_iter().map(|c| c.into()).collect::<Vec<_>>();
    
    let card_group = repo.create(CreateCardGroupDto {
        user: user.into(),
        title: title.into(),
        importance: importance.unwrap_or(0),
        tags,
        cards,
        difficulty: difficulty.unwrap_or(0),
    }).await?;
    
    Ok(card_group)
}
