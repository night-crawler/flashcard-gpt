use crate::command::CommandExt;
use crate::db::repositories::Repositories;
use crate::ext::card::ExtractValueExt;
use crate::ext::markdown::MarkdownFormatter;
use crate::ext::menu_repr::IteratorMenuReprExt;
use crate::message_render::RenderMessageTextHelper;
use crate::state::FlashGptDialogue;
use crate::state::{State, StateDescription};
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::dto::card::CardDto;
use flashcard_gpt_core::dto::card_group::CardGroupDto;
use flashcard_gpt_core::dto::tag::TagDto;
use itertools::Itertools;
use std::fmt::Debug;
use std::str::pattern::Pattern;
use std::str::FromStr;
use std::sync::Arc;
use teloxide::adaptors::DefaultParseMode;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, RequestError};
use tracing::{warn, Span};

static DIGITS: [&str; 11] = [
    "0️⃣", "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟",
];

#[derive(Debug, Clone)]
pub struct ChatManager {
    pub repositories: Repositories,
    pub formatter: MarkdownFormatter,
    pub binding: Arc<BindingDto>,
    pub bot: DefaultParseMode<Bot>,
    pub dialogue: FlashGptDialogue,
    pub message: Option<Arc<Message>>,
    pub span: Span,
}

impl ChatManager {
    pub async fn update_state(&self, next_state: State) -> anyhow::Result<StateDescription> {
        let desc = next_state.get_state_description(self.message.as_deref());
        self.dialogue.update(next_state).await?;
        Ok(desc)
    }

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
        text = ?text,
    ))]
    pub async fn send_message(
        &self,
        text: impl Into<String> + Debug,
    ) -> Result<Message, RequestError> {
        self.bot.send_message(self.dialogue.chat_id(), text).await
    }

    pub async fn get_state(&self) -> anyhow::Result<State> {
        Ok(self.dialogue.get_or_default().await?)
    }

    pub async fn get_description(&self) -> anyhow::Result<StateDescription> {
        Ok(self
            .get_state()
            .await?
            .get_state_description(self.message.as_deref()))
    }

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
    ))]
    pub async fn send_invalid_input(&self) -> anyhow::Result<()> {
        let desc = self.get_state_description().await?;
        self.send_message(desc.invalid_input.clone().as_ref())
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
    ))]
    pub async fn send_tag_menu(&self) -> anyhow::Result<()> {
        let desc = self.get_description().await?;
        let tag_menu = self
            .repositories
            .build_tag_menu(self.binding.user.id.clone())
            .await?;

        let combined = format!("{}\n{}", desc.repr, desc.prompt);
        self.bot
            .send_message(self.dialogue.chat_id(), combined)
            .reply_markup(tag_menu)
            .await?;

        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
    ))]
    pub async fn send_deck_menu(&self) -> anyhow::Result<()> {
        let desc = self.get_description().await?;
        let tag_menu = self
            .repositories
            .build_deck_menu(self.binding.user.id.clone())
            .await?;

        let combined = format!("{}\n\n{}", desc.repr, desc.prompt);
        self.bot
            .send_message(self.dialogue.chat_id(), combined)
            .reply_markup(tag_menu)
            .await?;

        Ok(())
    }

    pub async fn send_state_and_prompt(&self) -> anyhow::Result<()> {
        let desc = self.get_description().await?;
        let combined = format!("{}\n\n{}", desc.repr, desc.prompt);
        self.send_message(combined).await?;
        Ok(())
    }

    pub fn parse_html_values(&self, p: impl Pattern) -> Option<Vec<Arc<str>>> {
        self.message
            .as_deref()
            .and_then(|message| message.html_text())
            .map(|text| {
                text.split(p)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(Arc::from)
                    .collect()
            })
    }

    pub fn parse_html(&self) -> Option<Arc<str>> {
        self.message.as_deref()?.html_text().map(Arc::from)
    }

    pub fn parse_text(&self) -> Option<Arc<str>> {
        self.message.as_deref()?.text().map(Arc::from)
    }

    pub fn parse_integer<T>(&self) -> Option<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        match self
            .message
            .as_deref()
            .and_then(|message| message.text().map(|text| text.parse::<T>()))?
        {
            Ok(result) => Some(result),
            Err(err) => {
                warn!(?err, "Failed to parse integer");
                None
            }
        }
    }

    pub async fn send_help<T>(&self) -> anyhow::Result<()>
    where
        T: BotCommands,
    {
        self.send_message(T::descriptions().to_string()).await?;
        Ok(())
    }

    pub async fn send_menu<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        let menu = T::get_menu_items().into_menu_repr();
        self.bot
            .send_message(self.dialogue.chat_id(), T::get_menu_name())
            .reply_markup(menu)
            .await?;
        Ok(())
    }

    pub async fn delete_current_message(&self) -> anyhow::Result<()> {
        if let Some(message) = self.message.as_deref() {
            self.bot
                .delete_message(self.dialogue.chat_id(), message.id)
                .await?;
        }
        Ok(())
    }

    pub async fn set_menu_state<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        self.dialogue.update(T::get_corresponding_state()).await?;
        Ok(())
    }

    pub async fn set_my_commands<T>(&self) -> anyhow::Result<()>
    where
        T: BotCommands,
    {
        self.bot.set_my_commands(T::bot_commands()).await?;
        Ok(())
    }

    pub async fn get_state_description(&self) -> anyhow::Result<StateDescription> {
        let desc = self
            .get_state()
            .await?
            .get_state_description(self.message.as_deref());
        Ok(desc)
    }

    fn serialize_tags(
        &self,
        tags: impl IntoIterator<Item = impl AsRef<TagDto>>,
    ) -> anyhow::Result<String> {
        let tags = tags
            .into_iter()
            .map(|tag| self.formatter.to_html(&tag.as_ref().slug.replace("-", "_")))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|tag| format!("#{tag}"))
            .join(" ");
        Ok(tags)
    }

    pub async fn send_card_group(&self, cg: &CardGroupDto) -> anyhow::Result<()> {
        let title = format!("<b>{}</b>", self.formatter.to_html(cg.title.as_ref())?);
        let title = if let Some(link) = cg.extract_str("leetcode_link") {
            format!(r#"<a href="{}">{title}</a>"#, link)
        } else {
            title
        };

        let tags = self.serialize_tags(&cg.tags)?;

        let stats = format!(
            "Difficulty: {} Importance: {}",
            DIGITS[cg.difficulty as usize % 11],
            DIGITS[cg.importance as usize % 11],
        );

        let message = format!("[front] {title}\n\n{stats}\n\n{tags}");
        self.send_message(message).await?;

        Ok(())
    }

    pub async fn send_card(&self, card: &CardDto) -> anyhow::Result<()> {
        let title = format!("<b>{}</b>", self.formatter.to_html(card.title.as_ref())?);
        let title = if let Some(link) = card.extract_str("leetcode_link") {
            format!(r#"<a href="{}">{title}</a>"#, link)
        } else {
            title
        };

        let front = if let Some(front) = card.front.as_ref() {
            self.formatter.to_html(front.as_ref())?
        } else {
            String::new()
        };
        let back = if let Some(back) = card.back.as_ref() {
            self.formatter.to_html(back.as_ref())?
        } else {
            String::new()
        };
        let hints = card
            .hints
            .iter()
            .map(|hint| self.formatter.to_html(hint.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;

        let tags = self.serialize_tags(&card.tags)?;

        let stats = format!(
            "Difficulty: {} Importance: {}",
            DIGITS[card.difficulty as usize % 11],
            DIGITS[card.importance as usize % 11],
        );

        let front_message = format!("[front] {title}\n\n{stats}\n\n{front}\n\n{tags}");
        let hint_messages = hints
            .iter()
            .enumerate()
            .map(|(i, hint)| {
                format!(
                    "<blockquote expandable>Expand hint {}\n\n\n{hint}</blockquote>",
                    i + 1
                )
            })
            .join("\n");
        let back = format!("<tg-spoiler>{back}</tg-spoiler>");

        self.send_message(front_message).await?;
        self.send_message(hint_messages).await?;
        self.send_message(back).await?;

        Ok(())
    }
}
