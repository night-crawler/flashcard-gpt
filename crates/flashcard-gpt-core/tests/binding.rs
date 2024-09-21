use flashcard_gpt_core::dto::binding::GetOrCreateBindingDto;
use flashcard_gpt_core::repo::binding::BindingRepo;
use flashcard_gpt_tests::db::{TestDbExt, TEST_DB};
use serde_json::json;
use testresult::TestResult;

#[tokio::test]
async fn test_get_or_create_binding() -> TestResult {
    let db = TEST_DB.get_client().await?;
    let repo = BindingRepo::new_binding(db, tracing::span!(tracing::Level::INFO, "test"), true);

    let result = repo.get_by_source_id("source_id".into()).await?;
    assert!(result.is_none());

    let dto = GetOrCreateBindingDto {
        source_id: "source_id".into(),
        type_name: "sample".into(),
        email: "qemail@email.com".into(),
        name: "name".into(),
        password: "password".into(),
        data: json!({
            "a": "b"
        })
        .into(),
    };
    let binding = repo.get_or_create_binding(dto).await?;
    assert_eq!(binding.source_id.as_ref(), "source_id");

    Ok(())
}
