use crate::ext::json_value::ValueExt;
use flashcard_gpt_core::dto::card::CardDto;
use flashcard_gpt_core::dto::card_group::CardGroupDto;
use serde_json::Value;
use flashcard_gpt_core::dto::binding::BindingDto;

pub trait ExtractValueExt {
    fn extract_value(&self, key: &str) -> Option<&Value>;
    fn extract_str(&self, key: &str) -> Option<&str>;
}

impl ExtractValueExt for CardDto {
    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.data.as_ref()?.get_value_by(key)
    }

    fn extract_str(&self, key: &str) -> Option<&str> {
        self.extract_value(key)?.as_str()
    }
}

impl ExtractValueExt for CardGroupDto {
    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.data.as_ref()?.get_value_by(key)
    }

    fn extract_str(&self, key: &str) -> Option<&str> {
        self.extract_value(key)?.as_str()
    }
}

impl ExtractValueExt for BindingDto {
    fn extract_value(&self, key: &str) -> Option<&Value> {
        self.data.as_ref()?.get_value_by(key)
    }

    fn extract_str(&self, key: &str) -> Option<&str> {
        self.extract_value(key)?.as_str()
    }
}
