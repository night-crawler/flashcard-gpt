use crate::db::surreal_test_container::{SurrealDbTestContainer, SURREALDB_PORT};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testresult::TestResult;
use tracing::{error, info};

pub struct TestDb {
    pub container: ContainerAsync<SurrealDbTestContainer>,
}

impl TestDb {
    pub async fn new() -> TestResult<Self> {
        let container = prepare_database().await?;
        Ok(Self { container })
    }
}

pub async fn prepare_database() -> TestResult<ContainerAsync<SurrealDbTestContainer>> {
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

    let migration_data = include_str!(
        "../../../flashcard-gpt-core/db-migrations/migrations/20240902_185441_Initial.surql"
    );
    let mut response = db.query(migration_data).await?;

    let mut last_error = None;

    for (id, err) in response.take_errors() {
        error!(%id, ?err, "Query failed");
        last_error = Some(err);
    }

    if let Some(err) = last_error {
        Err(err)?;
    }

    info!("Migration complete");

    Ok(node)
}
