use crate::dto::time::Time;
use crate::dto::user::User;
use crate::reexports::db::sql::Thing;
use bon::Builder;
use chrono::{DateTime, NaiveTime};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Duration;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct GlobalSettingsDto {
    pub id: Thing,
    pub daily_limit: u16,
    pub timetable: Vec<[Duration; 2]>,
    pub timezone: Tz,
    pub user: User,
    pub time: Time,
}

impl GlobalSettingsDto {
    pub fn ts_matches(&self, now: DateTime<Tz>) -> bool {
        let current_time = now.time();
        let now_duration = current_time
            - NaiveTime::parse_from_str("00:00:00", "%H:%M:%S")
                .expect("Failed to parse 00:00:00 time, it must never happen");
        for &[start, end] in self.timetable.iter() {
            let (start, end) = (
                chrono::Duration::from_std(start.0).unwrap(),
                chrono::Duration::from_std(end.0).unwrap(),
            );
            if now_duration >= start && now_duration <= end {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateGlobalSettingsDto {
    pub user: Thing,
    pub daily_limit: u16,
    pub timetable: Vec<[Duration; 2]>,
    pub timezone: Tz,
}

impl From<GlobalSettingsDto> for Thing {
    fn from(value: GlobalSettingsDto) -> Self {
        value.id
    }
}

impl From<&GlobalSettingsDto> for Thing {
    fn from(value: &GlobalSettingsDto) -> Self {
        value.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Tz;
    use std::sync::Arc;
    use testresult::TestResult;

    fn build_test_settings(durations: Vec<[Duration; 2]>) -> GlobalSettingsDto {
        GlobalSettingsDto {
            id: Thing::from(("test_user", "aaa")),
            daily_limit: 100,
            time: Time::default(),
            timetable: durations,
            timezone: Tz::Europe__Dublin,
            user: User {
                id: Thing::from(("test_user", "aaa")),
                email: Arc::from("aaa@aaa.aa"),
                name: Arc::from("aaa"),
                password: Arc::from("aaa"),
                time: None,
            },
        }
    }
    #[test]
    fn test_is_within() -> TestResult {
        let now = DateTime::parse_from_rfc3339("2021-02-13T15:30:00Z")?.to_utc();
        let now = now.with_timezone(&Tz::UTC);
        let settings = build_test_settings(vec![
            [Duration::from_hours(15), Duration::from_hours(16)],
            [Duration::from_mins(0), Duration::from_hours(3)],
        ]);

        assert!(settings.ts_matches(now));

        Ok(())
    }
}
