use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::marker::PhantomData;
use chrono::Duration;
use humantime::{format_duration, parse_duration};

pub mod binding;
pub mod card;
pub mod card_group;
pub mod deck;
pub mod deck_card;
pub mod deck_card_group;
pub mod global_settings;
pub mod history;
pub mod llm;
pub mod tag;
pub mod time;
pub mod user;

fn skip_nulls<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct SkipNulls<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for SkipNulls<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("array with nulls")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<Option<T>>()? {
                vec.extend(elem);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_seq(SkipNulls(PhantomData))
}


fn from_raw_durations<'de, D, const N: usize>(deserializer: D) -> Result<Vec<[Duration; N]>, D::Error>
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

fn to_raw_durations<S, const N: usize>(value: &[[Duration; N]], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s: Vec<Vec<String>> = value
        .iter()
        .map(|inner| {
            inner
                .iter()
                .map(|dur| {
                    let mut formatted = format_duration(dur.to_std().unwrap()).to_string();
                    formatted.retain(|c| !c.is_whitespace());
                    formatted
                })
                .collect()
        })
        .collect();
    s.serialize(serializer)
}


#[allow(dead_code)]
fn from_raw_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let dur_str: String = Deserialize::deserialize(deserializer)?;
    let duration = parse_duration(&dur_str)
        .map_err(|err| Error::custom(format_args!("failed to parse duration: {}", err)))?;
    let duration = Duration::from_std(duration).map_err(|err| {
        Error::custom(format_args!("failed to convert duration: {}", err))
    })?;
    Ok(duration)
}


fn to_raw_duration<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut formatted = format_duration(value.to_std().unwrap()).to_string();
    formatted.retain(|c| !c.is_whitespace());
    formatted.serialize(serializer)
}

fn to_raw_duration_option<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let Some(value) = value else {
        return serializer.serialize_none();
    };

    to_raw_duration(value, serializer)
}

fn from_raw_duration_option<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let dur_str: Option<String> = Deserialize::deserialize(deserializer)?;
    let Some(dur_str) = dur_str else {
        return Ok(None);
    };
    
    let duration = parse_duration(&dur_str)
        .map_err(|err| Error::custom(format_args!("failed to parse duration: {}", err)))?;
    let duration = Duration::from_std(duration).map_err(|err| {
        Error::custom(format_args!("failed to convert duration: {}", err))
    })?;
    Ok(Some(duration))
}
