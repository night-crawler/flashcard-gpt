use flashcard_gpt_core::dto::binding::GetOrCreateBindingDto;
use flashcard_gpt_tests::db::utils::create_binding_repo;
use serde_json::json;
use testresult::TestResult;

#[tokio::test]
async fn test_get_or_create_binding() -> TestResult {
    let repo = create_binding_repo().await?;
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
    
    let banned = repo.set_banned(&binding).await?;
    println!("{:?}", banned);

    Ok(())
}
