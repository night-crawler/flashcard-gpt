use crate::dto::user::User;
use crate::error::CoreError;
use crate::ext::mutex::MutexExt;
use crate::repo::user::UserRepo;
use crate::tests::surreal_test_container::{SurrealDbTestContainer, SURREALDB_PORT};
use crate::tests::TEST_DB;
use log::info;
use std::sync::Mutex;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;

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


pub async fn prepare_database() -> Result<(ContainerAsync<SurrealDbTestContainer>, Surreal<Client>), CoreError> {
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

    let migration_data = include_str!("../../db-migrations/migrations/20240902_185441_Initial.surql");
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
    let repo = UserRepo::new(db).await;

    let user = repo.create_user(User {
        id: None,
        email: format!("{}@example.com", name.to_lowercase()).into(),
        name: name.to_string().into(),
        password: name.to_string().into(),
        time: None,
    }).await?;

    Ok(user)
}
