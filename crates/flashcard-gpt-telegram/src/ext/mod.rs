use anyhow::anyhow;
use flashcard_gpt_core::reexports::db::sql::Thing;

pub mod binding;
pub mod bot;
pub mod dialogue;
pub mod menu_repr;
pub mod message;
pub mod rendering;


pub trait StrExt {
    fn as_thing(&self) -> anyhow::Result<Thing>;
}

impl StrExt for str {
    fn as_thing(&self) -> anyhow::Result<Thing> {
        let thing = Thing::try_from(self).map_err(|_| anyhow!("Failed to build Thing from {self}"))?;
        Ok(thing)
    }
}
