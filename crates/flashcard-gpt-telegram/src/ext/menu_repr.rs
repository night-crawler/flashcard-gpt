use flashcard_gpt_core::dto::deck::DeckDto;
use flashcard_gpt_core::dto::tag::TagDto;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub trait MenuReprExt {
    fn menu_repr(&self) -> InlineKeyboardButton;
}

impl MenuReprExt for TagDto {
    fn menu_repr(&self) -> InlineKeyboardButton {
        InlineKeyboardButton::callback(self.name.as_ref(), self.slug.as_ref())
    }
}

impl MenuReprExt for DeckDto {
    fn menu_repr(&self) -> InlineKeyboardButton {
        InlineKeyboardButton::callback(self.title.as_ref(), self.id.to_string())
    }
}

impl MenuReprExt for InlineKeyboardButton {
    fn menu_repr(&self) -> InlineKeyboardButton {
        self.clone()
    }
}

pub trait IteratorMenuReprExt {
    fn into_menu_repr(self) -> InlineKeyboardMarkup;
}

impl<I, T> IteratorMenuReprExt for I
where
    I: Iterator<Item = T>,
    T: MenuReprExt,
{
    fn into_menu_repr(self) -> InlineKeyboardMarkup {
        build_menu(self)
    }
}

pub fn build_menu<T>(items: impl Iterator<Item = T>) -> InlineKeyboardMarkup
where
    T: MenuReprExt,
{
    let mut rows = vec![vec![]];
    let mut current_length = 0;

    for item in items {
        if current_length >= 15 {
            rows.push(vec![]);
            current_length = 0;
        }
        let repr = item.menu_repr();
        current_length += repr.text.len();
        rows.last_mut().unwrap().push(repr);
    }

    InlineKeyboardMarkup::new(rows)
}
