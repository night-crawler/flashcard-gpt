use crate::tests::utils::TestDb;
use ctor::{ctor, dtor};
use std::future::Future;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::select;

pub mod surreal_test_container;
pub mod utils;

use crate::error::CoreError;
use crate::logging::init_tracing;
use crate::tests::surreal_test_container::SURREALDB_PORT;
use tokio::sync::OnceCell;
use tracing::{error, info};

pub static TEST_DB: OnceCell<TestDb> = OnceCell::const_new();

#[ctor]
fn initialize_tracing() {
    if let Err(e) = init_tracing() {
        eprintln!("Error initializing tracking: {e:?}");
    }
}

#[dtor]
fn kill_container() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        info!("Stopping SurrealDb container");
        if let Some(t) = TEST_DB.get() {
            select! {
                res = t.container.stop() => match res {
                    Ok(_) => {}
                    Err(e) => error!("Error stopping container: {e:?}"),
                },
                // eventually it will stop
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {}
            }
            info!("Stopped SurrealDb container");
        }
    });
    rt.shutdown_background();
}

pub trait TestDbExt {
    fn get_client(&self) -> impl Future<Output = Result<Surreal<Client>, CoreError>>;
}

impl TestDbExt for OnceCell<TestDb> {
    async fn get_client(&self) -> Result<Surreal<Client>, CoreError> {
        let db = self.get_or_try_init(TestDb::new).await?;

        let host_port = db.container.get_host_port_ipv4(SURREALDB_PORT).await?;
        let url = format!("127.0.0.1:{host_port}");

        let db: Surreal<Client> = Surreal::init();
        db.connect::<Ws>(url).await?;
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        db.use_ns("test").use_db("test").await?;

        Ok(db)
    }
}
