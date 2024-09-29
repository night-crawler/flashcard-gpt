use crate::dto::time::Time;
use crate::dto::user::User;
use crate::reexports::db::sql::Thing;
use bon::Builder;
use chrono::{DateTime, Duration, NaiveTime};
use chrono_tz::Tz;
use humantime::format_duration;
use humantime::parse_duration;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct GlobalSettingsDto {
    pub id: Thing,
    pub daily_limit: u16,
    #[serde(deserialize_with = "from_raw", serialize_with = "to_raw")]
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
    #[serde(deserialize_with = "from_raw", serialize_with = "to_raw")]
    pub timetable: Vec<[Duration; 2]>,
    pub timezone: Tz,
}

fn from_raw<'de, D, const N: usize>(deserializer: D) -> Result<Vec<[Duration; N]>, D::Error>
where
    D: Deserializer<'de>,
{
    let pairs: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;
    let mut result = vec![];

    for pair in pairs {
        if pair.len() != N {
            return Err(Error::custom(format_args!(
                "expected a pair of durations, got {}",
                pair.len()
            )));
        }
        let mut inner = [Duration::zero(); N];
        for (i, dur_str) in pair.into_iter().enumerate() {
            let duration = parse_duration(&dur_str)
                .map_err(|err| Error::custom(format_args!("failed to parse duration: {}", err)))?;
            let duration = Duration::from_std(duration).map_err(|err| {
                Error::custom(format_args!("failed to convert duration: {}", err))
            })?;
            inner[i] = duration;
        }
        result.push(inner);
    }

    Ok(result)
}

fn to_raw<S, const N: usize>(value: &[[Duration; N]], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s: Vec<Vec<String>> = value
        .iter()
        .map(|inner| {
            inner
                .iter()
                .map(|dur| format_duration(dur.to_std().unwrap()).to_string())
                .collect()
        })
        .collect();
    s.serialize(serializer)
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
    use serde_json;
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
    fn test_serialize() -> TestResult {
        let settings = build_test_settings(vec![
            [Duration::minutes(30), Duration::hours(1)],
            [Duration::minutes(2), Duration::hours(3)],
        ]);

        let serialized = serde_json::to_string_pretty(&settings)?;

        assert!(serialized.contains("30m"));
        assert!(serialized.contains("1h"));
        assert!(serialized.contains("2m"));
        assert!(serialized.contains("3h"));

        let deserialized: GlobalSettingsDto = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.timetable[0][0], Duration::minutes(30));
        assert_eq!(deserialized.timetable[0][1], Duration::hours(1));
        assert_eq!(deserialized.timetable[1][0], Duration::minutes(2));
        assert_eq!(deserialized.timetable[1][1], Duration::hours(3));

        Ok(())
    }

    #[test]
    fn test_is_within() -> TestResult {
        let now = DateTime::parse_from_rfc3339("2021-02-13T15:30:00Z")?.to_utc();
        let now= now.with_timezone(&Tz::UTC);
        let settings = build_test_settings(vec![
            [Duration::hours(15), Duration::hours(16)],
            [Duration::minutes(0), Duration::hours(3)],
        ]);

        assert!(settings.ts_matches(now));

        Ok(())
    }
}
