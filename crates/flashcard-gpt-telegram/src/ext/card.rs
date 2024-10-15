use crate::ext::json_value::ValueExt;
use flashcard_gpt_core::model::binding::Binding;
use flashcard_gpt_core::model::card::Card;
use flashcard_gpt_core::model::card_group::CardGroup;
use serde_json::Value;

pub trait ExtractValueExt {
    fn extract_value(&self, key: &str) -> Option<&Value>;
    fn extract_str(&self, key: &str) -> Option<&str>;
}

impl ExtractValueExt for Card {
    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.data.as_ref()?.get_value_by(key)
    }

    fn extract_str(&self, key: &str) -> Option<&str> {
        self.extract_value(key)?.as_str()
    }
}

impl ExtractValueExt for CardGroup {
    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.data.as_ref()?.get_value_by(key)
    }

    fn extract_str(&self, key: &str) -> Option<&str> {
        self.extract_value(key)?.as_str()
    }
}

impl ExtractValueExt for Binding {
    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.data.as_ref()?.get_value_by(key)
    }

    fn extract_str(&self, key: &str) -> Option<&str> {
        self.extract_value(key)?.as_str()
    }
}
