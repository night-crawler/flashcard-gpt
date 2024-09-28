use chrono::Duration;
use flashcard_gpt_core::dto::global_settings::CreateGlobalSettingsDto;
use flashcard_gpt_tests::db::utils::{create_global_settings_repo, create_user};
use testresult::TestResult;

#[tokio::test]
async fn test_create() -> TestResult {
    let repo = create_global_settings_repo().await?;
    let user = create_user("global_settings_create").await?;
    let settings = repo
        .create_custom(CreateGlobalSettingsDto {
            user: user.id.clone(),
            daily_limit: 88,
            timetable: vec![
                [Duration::hours(10), Duration::hours(11)],
                [Duration::hours(13), Duration::hours(14)],
                [Duration::hours(17), Duration::hours(18)],
            ],
        })
        .await?;

    assert_eq!(settings.daily_limit, 88);
    assert_eq!(
        settings.timetable,
        vec![
            [Duration::hours(10), Duration::hours(11)],
            [Duration::hours(13), Duration::hours(14)],
            [Duration::hours(17), Duration::hours(18)],
        ]
    );

    // second create for the same user must fail
    let result = repo
        .create_custom(CreateGlobalSettingsDto {
            user: user.id.clone(),
            daily_limit: 88,
            timetable: vec![
                [Duration::hours(10), Duration::hours(11)],
                [Duration::hours(13), Duration::hours(14)],
                [Duration::hours(17), Duration::hours(18)],
            ],
        })
        .await;
    assert!(result.is_err());

    Ok(())
}
