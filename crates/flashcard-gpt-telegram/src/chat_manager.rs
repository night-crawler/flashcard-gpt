use crate::command::answer::AnswerCommand;
use crate::command::ext::CommandExt;
use crate::db::repositories::Repositories;
use crate::ext::binding::ChatIdExt;
use crate::ext::card::ExtractValueExt;
use crate::ext::json_value::ValueExt;
use crate::ext::markdown::MarkdownFormatter;
use crate::ext::menu_repr::IteratorMenuReprExt;
use crate::message_render::RenderMessageTextHelper;
use crate::state::bot_state::{BotState, FlashGptDialogue};
use crate::state::state_description::StateDescription;
use crate::state::state_fields::StateFields;
use anyhow::bail;
use chrono::{TimeDelta, Utc};
use flashcard_gpt_core::dto::binding::BindingDto;
use flashcard_gpt_core::dto::card::{CardDto, UpdateCardDto};
use flashcard_gpt_core::dto::card_group::{CardGroupDto, UpdateCardGroupDto};
use flashcard_gpt_core::dto::history::CreateHistoryDto;
use flashcard_gpt_core::dto::tag::TagDto;
use flashcard_gpt_core::dto::user::User;
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use flashcard_gpt_core::reexports::db::sql::{Duration, Thing};
use itertools::Itertools;
use rand::Rng;
use std::fmt::Debug;
use std::ops::Sub;
use std::str::pattern::Pattern;
use std::str::FromStr;
use std::sync::Arc;
use teloxide::adaptors::DefaultParseMode;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;
use tracing::{debug, warn, Span};

static DIGITS: [&str; 11] = [
    "0️⃣", "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟",
];

#[derive(Debug, Clone)]
pub struct ChatManager {
    pub repo: Repositories,
    pub generator: CardGeneratorService,
    pub formatter: MarkdownFormatter,
    pub binding: Arc<BindingDto>,
    pub bot: DefaultParseMode<Bot>,
    pub dialogue: FlashGptDialogue,
    pub message: Option<Arc<Message>>,
    pub span: Span,
}

impl ChatManager {
    pub async fn update_state(&self, next_state: BotState) -> anyhow::Result<StateDescription> {
        let desc = next_state.get_state_description(self.message.as_deref());
        self.dialogue.update(next_state).await?;
        Ok(desc)
    }

    #[tracing::instrument(level = "info", skip_all, parent = &self.span, err, fields(
        chat_id = ?self.dialogue.chat_id(),
        message = ?self.message,
        text = ?text,
    ))]
    pub async fn send_message(&self, text: impl Into<String> + Debug) -> anyhow::Result<Message> {
        let text = text.into();
        if text.is_empty() {
            bail!("Tried to send an empty message");
        }

        let mut last_message = None;
        for chunk in Self::split_html(&text)? {
            if chunk.trim().is_empty() {
                continue;
            }
            last_message = self
                .bot
                .send_message(self.dialogue.chat_id(), chunk)
                .await?
                .into();
        }

        Ok(last_message.unwrap())
    }

    pub async fn send_markdown_message(
        &self,
        text: impl Into<String> + Debug,
    ) -> anyhow::Result<Message> {
        let text = text.into();
        let text = self.formatter.to_html(&text)?;
        self.send_message(text).await
    }

    pub async fn get_state(&self) -> anyhow::Result<BotState> {
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
            .repo
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
            .repo
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

    pub async fn send_customized_menu<T>(
        &self,
        cb: impl FnOnce(InlineKeyboardMarkup) -> InlineKeyboardMarkup,
    ) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        let menu = T::get_menu_items().into_menu_repr();
        let menu = cb(menu);
        self.bot
            .send_message(self.dialogue.chat_id(), T::get_menu_name())
            .reply_markup(menu)
            .await?;
        Ok(())
    }

    pub async fn send_menu<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        self.send_customized_menu::<T>(|kb| kb).await
    }

    pub async fn send_answer_menu(&self) -> anyhow::Result<()> {
        self.send_customized_menu::<AnswerCommand>(|kb| {
            let range_buttons_top = (0..5).map(|difficulty| {
                InlineKeyboardButton::callback(difficulty.to_string(), difficulty.to_string())
            });
            let range_buttons_down = (5..11).map(|difficulty| {
                InlineKeyboardButton::callback(difficulty.to_string(), difficulty.to_string())
            });
            kb.append_row(range_buttons_top)
                .append_row(range_buttons_down)
        })
        .await
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
            let back = self.formatter.to_html(back.as_ref())?;
            Some(format!("<tg-spoiler>{back}</tg-spoiler>"))
        } else {
            None
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

        self.send_message(front_message).await?;
        if !hints.is_empty() {
            self.send_message(hint_messages).await?;
        } else {
            warn!(?card, "No hint message");
        }
        if let Some(back) = back {
            self.send_message(back).await?;
        }

        Ok(())
    }

    fn split_html(text: impl AsRef<str>) -> anyhow::Result<Vec<String>> {
        let no_split = &["a"];
        let text = text.as_ref();
        for chunk_size in (1000..4000).rev() {
            match dumb_html_splitter::split(text, chunk_size, no_split) {
                Ok(chunks) => return Ok(chunks),
                Err(err) => {
                    warn!(%text, %err, "Failed to split text")
                }
            }
        }

        bail!("Failed to split text {text}")
    }

    pub async fn commit_answer(
        &self,
        difficulty: u8,
        hide_for: Option<Duration>,
    ) -> anyhow::Result<()> {
        let fields = self.get_state().await?.into_fields();
        if let Some(Some(dcg_id)) = fields.deck_card_group_id() {
            self.repo
                .history
                .create_custom(CreateHistoryDto {
                    user: self.binding.user.id.clone(),
                    deck_card: None,
                    deck_card_group: dcg_id.clone().into(),
                    difficulty,
                    time: None,
                    hide_for,
                })
                .await?;
            return Ok(());
        }

        if let Some(Some(dc_id)) = fields.deck_card_id() {
            self.repo
                .history
                .create_custom(CreateHistoryDto {
                    user: self.binding.user.id.clone(),
                    deck_card: dc_id.clone().into(),
                    deck_card_group: None,
                    difficulty,
                    time: None,
                    hide_for,
                })
                .await?;
            return Ok(());
        }

        bail!("No active deck card or deck card group in the state");
    }

    pub fn get_user(&self) -> &User {
        self.binding.user.as_ref()
    }

    pub fn get_user_id(&self) -> &Thing {
        &self.get_user().id
    }
}

impl ChatManager {
    pub async fn answer_with_card(&self) -> anyhow::Result<bool> {
        let user = self.get_user();
        let chat_id = self.binding.get_chat_id()?;

        let now = Utc::now();
        let past_3h = now.sub(TimeDelta::hours(3));

        let mut dcs = self
            .repo
            .decks
            .list_top_ranked_cards(user, past_3h.to_utc())
            .await?;
        if dcs.is_empty() {
            debug!(%user, %chat_id, "No deck cards to display");
            return Ok(false);
        }
        let id = rand::thread_rng().gen_range(0..dcs.len());
        let dc = dcs.swap_remove(id);
        self.update_state(BotState::Answering(StateFields::Answer {
            deck_card_group_id: None,
            deck_card_group_card_seq: None,
            deck_card_id: Some(dc.id),
            difficulty: None,
        }))
        .await?;
        self.send_card(dc.card.as_ref()).await?;

        Ok(true)
    }

    pub async fn answer_with_card_group(&self) -> anyhow::Result<bool> {
        let now = Utc::now();
        let past_3h = now.sub(TimeDelta::hours(3));

        let user = self.get_user();
        let chat_id = self.binding.get_chat_id()?;

        let mut dcgs = self
            .repo
            .decks
            .list_top_ranked_card_groups(user, past_3h.to_utc())
            .await?;
        if dcgs.is_empty() {
            debug!(%user, %chat_id, "No deck card groups to display");
            return Ok(false);
        }
        let card_id = rand::thread_rng().gen_range(0..dcgs.len());
        let dcg = dcgs.swap_remove(card_id);
        self.update_state(BotState::Answering(StateFields::Answer {
            deck_card_group_id: Some(dcg.id),
            deck_card_group_card_seq: Some(0),
            deck_card_id: None,
            difficulty: None,
        }))
        .await?;
        self.send_card_group(dcg.card_group.as_ref()).await?;
        self.send_card(dcg.card_group.cards[0].as_ref()).await?;

        Ok(true)
    }

    pub async fn send_card_group_data_by_key(
        &self,
        id: &Thing,
        key: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let key = key.as_ref();

        let data = match id.tb.as_str() {
            "deck_card_group" => self
                .repo
                .decks
                .get_deck_card_group(id.clone())
                .await?
                .card_group
                .data
                .clone(),
            "deck_card" => self
                .repo
                .decks
                .get_deck_card(id.clone())
                .await?
                .card
                .data
                .clone(),
            _ => {
                bail!("Provided an unsupported id: {id}")
            }
        };

        let Some(data) = data else {
            warn!(?id, "No data found");
            self.send_message("No data in the data field in this Card or Card Group".to_string())
                .await?;
            return Ok(());
        };

        let Some(value) = data.get_value_by(key) else {
            self.send_message(format!("Card group data does not contain key {key}"))
                .await?;
            return Ok(());
        };

        let Some(value) = value.as_str() else {
            self.send_message("Cannot represent the given data value as string")
                .await?;
            return Ok(());
        };

        self.send_markdown_message(value).await?;

        Ok(())
    }

    pub async fn update_deck_card_group_inner(
        &self,
        dcg_id: impl Into<Thing>,
        update_card_group_dto: UpdateCardGroupDto,
    ) -> anyhow::Result<CardGroupDto> {
        let cg_id = self
            .repo
            .decks
            .get_deck_card_group(dcg_id)
            .await?
            .card_group
            .id
            .clone();
        let cg = self
            .repo
            .card_groups
            .patch(cg_id, update_card_group_dto)
            .await?;
        Ok(cg)
    }

    pub async fn update_deck_card_inner(
        self,
        dc_id: impl Into<Thing>,
        update_card_dto: UpdateCardDto,
    ) -> anyhow::Result<CardDto> {
        let card_id = self.repo.decks.get_deck_card(dc_id).await?.card.id.clone();
        let cg = self.repo.cards.patch(card_id, update_card_dto).await?;
        Ok(cg)
    }
}
