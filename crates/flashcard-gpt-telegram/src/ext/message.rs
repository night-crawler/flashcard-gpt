use teloxide::prelude::Message;

pub trait MessageExt {
    fn get_text(&self) -> &str;
}

impl MessageExt for Message {
    fn get_text(&self) -> &str {
        self.text().unwrap_or_default()
    }
}