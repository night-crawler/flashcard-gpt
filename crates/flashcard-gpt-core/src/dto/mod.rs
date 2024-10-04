use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

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
