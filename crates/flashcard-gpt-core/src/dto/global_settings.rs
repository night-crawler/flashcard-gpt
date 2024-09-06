use crate::dto::time::Time;
use crate::dto::user::User;
use crate::reexports::db::sql::Thing;
use chrono::Duration;
use humantime::format_duration;
use humantime::parse_duration;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub id: Thing,
    pub daily_limit: i32,
    pub time: Time,
    #[serde(deserialize_with = "from_raw", serialize_with = "to_raw")]
    pub timetable: Vec<[Duration; 2]>,
    pub user: User,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::sync::Arc;
    use testresult::TestResult;

    #[test]
    fn test_serialize() -> TestResult {
        let durations = vec![
            [Duration::minutes(30), Duration::hours(1)],
            [Duration::minutes(2), Duration::hours(3)],
        ];
        let settings = GlobalSettings {
            id: Thing::from(("test_user", "aaa")),
            daily_limit: 100,
            time: Time::default(),
            timetable: durations,
            user: User {
                id: Thing::from(("test_user", "aaa")),
                email: Arc::from("aaa@aaa.aa"),
                name: Arc::from("aaa"),
                password: Arc::from("aaa"),
                time: None,
            },
        };

        let serialized = serde_json::to_string(&settings)?;

        assert!(serialized.contains("30m"));
        assert!(serialized.contains("1h"));
        assert!(serialized.contains("2m"));
        assert!(serialized.contains("3h"));

        let deserialized: GlobalSettings = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.timetable[0][0], Duration::minutes(30));
        assert_eq!(deserialized.timetable[0][1], Duration::hours(1));
        assert_eq!(deserialized.timetable[1][0], Duration::minutes(2));
        assert_eq!(deserialized.timetable[1][1], Duration::hours(3));

        Ok(())
    }
}
