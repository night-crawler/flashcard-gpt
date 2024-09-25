use anyhow::anyhow;
use flashcard_gpt_core::reexports::db::sql::Thing;
use itertools::Itertools;

pub mod binding;
pub mod bot;
pub mod card;
pub mod dialogue;
pub mod json_value;
pub mod markdown;
pub mod menu_repr;
pub mod message;
pub mod rendering;

pub trait StrExt {
    fn as_thing(&self) -> anyhow::Result<Thing>;
    fn filter_whitespace(&self) -> String;
}

impl StrExt for str {
    fn as_thing(&self) -> anyhow::Result<Thing> {
        let thing =
            Thing::try_from(self).map_err(|_| anyhow!("Failed to build Thing from {self}"))?;
        Ok(thing)
    }

    fn filter_whitespace(&self) -> String {
        self.split_whitespace().join(" ")
    }
}
