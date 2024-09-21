use crate::db::surreal_test_container::SURREALDB_PORT;
use crate::db::test_db::TestDb;
use ctor::dtor;
use std::future::Future;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use testresult::TestResult;
use tokio::select;
use tokio::sync::OnceCell;

pub mod surreal_test_container;
pub mod test_db;
pub mod utils;

pub static TEST_DB: OnceCell<TestDb> = OnceCell::const_new();

pub trait TestDbExt {
    fn get_client(&self) -> impl Future<Output = TestResult<Surreal<Client>>>;
}

impl TestDbExt for OnceCell<TestDb> {
    async fn get_client(&self) -> TestResult<Surreal<Client>> {
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

#[dtor]
fn kill_container() {
    let Ok(rt) = tokio::runtime::Runtime::new() else {
        return;
    };
    rt.block_on(async move {
        if let Some(t) = TEST_DB.get() {
            select! {
                _ = t.container.stop() => {}
                // eventually it will stop
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {}
            }
        }
    });
    rt.shutdown_background();
}
