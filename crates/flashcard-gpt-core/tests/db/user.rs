use flashcard_gpt_core::repo::user::UserRepo;
use flashcard_gpt_tests::db::utils::create_user;
use flashcard_gpt_tests::db::TestDbExt;
use flashcard_gpt_tests::db::TEST_DB;
use testresult::TestResult;
use tracing::{span, Level};

#[tokio::test]
async fn test_create_user() -> TestResult {
    let db = TEST_DB.get_client().await?;
    let repo = UserRepo::new_user(db, span!(Level::INFO, "user_create"), true);

    let _ = repo.list_users().await?;

    let user = create_user("Bla").await?;

    assert_eq!(user.email.as_ref(), "bla@example.com");
    assert_eq!(user.name.as_ref(), "Bla");

    assert!(!user.password.is_empty());
    assert!(user.time.is_some());

    let user = repo.get_by_id(user.id).await?;
    println!("{:?}", user);

    Ok(())
}
