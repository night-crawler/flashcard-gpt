use chrono_tz::Tz;
use flashcard_gpt_core::model::global_settings::CreateGlobalSettings;
use flashcard_gpt_tests::db::utils::{create_global_settings_repo, create_user};
use std::ops::Add;
use surrealdb::sql::Duration;
use testresult::TestResult;

#[tokio::test]
async fn test_create() -> TestResult {
    let repo = create_global_settings_repo().await?;
    let user = create_user("global_settings_create").await?;
    let one = Duration::from_mins(1)
        .add(Duration::from_secs(1))
        .add(Duration::from_millis(1));
    let settings = repo
        .create(CreateGlobalSettings {
            user: user.id.clone(),
            daily_limit: 88,
            timetable: vec![
                [Duration::from_hours(10).add(one), Duration::from_hours(11)],
                [Duration::from_hours(13), Duration::from_hours(14)],
                [Duration::from_hours(17), Duration::from_hours(18)],
            ],
            timezone: Tz::Europe__Dublin,
        })
        .await?;

    assert_eq!(settings.daily_limit, 88);
    assert_eq!(
        settings.timetable,
        vec![
            [Duration::from_hours(10).add(one), Duration::from_hours(11)],
            [Duration::from_hours(13), Duration::from_hours(14)],
            [Duration::from_hours(17), Duration::from_hours(18)],
        ]
    );

    // second create for the same user must fail
    let result = repo
        .create(CreateGlobalSettings {
            user: user.id.clone(),
            daily_limit: 88,
            timetable: vec![
                [Duration::from_hours(10), Duration::from_hours(11)],
                [Duration::from_hours(13), Duration::from_hours(14)],
                [Duration::from_hours(17), Duration::from_hours(18)],
            ],
            timezone: Tz::Europe__Dublin,
        })
        .await;
    assert!(result.is_err());

    Ok(())
}
