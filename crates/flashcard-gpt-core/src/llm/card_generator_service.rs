use crate::dto::card::CreateCardDto;
use crate::dto::card_group::CreateCardGroupDto;
use crate::dto::deck_card_group::{CreateDeckCardGroupDto, DeckCardGroupDto};
use crate::dto::llm::GptCardGroup;
use crate::error::CoreError;
use crate::llm::custom_executor::{CustomExecutor, CustomStep};
use crate::reexports::db::sql::Thing;
use crate::repo::card::CardRepo;
use crate::repo::card_group::CardGroupRepo;
use crate::repo::deck::DeckRepo;
use crate::repo::tag::TagRepo;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Clone)]
pub struct CardGeneratorService {
    pub card_generator: CustomExecutor,
    pub cards: CardRepo,
    pub card_groups: CardGroupRepo,
    pub decks: DeckRepo,
    pub tags: TagRepo,
}

impl Debug for CardGeneratorService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CardGeneratorService")
    }
}

impl CardGeneratorService {
    pub fn new(
        card_generator: CustomExecutor,
        cards: CardRepo,
        card_groups: CardGroupRepo,
        decks: DeckRepo,
        tags: TagRepo,
    ) -> Self {
        Self {
            card_generator,
            cards,
            card_groups,
            decks,
            tags,
        }
    }
    pub async fn generate_code_cards(
        &self,
        code: impl AsRef<str>,
    ) -> Result<(String, BTreeMap<Arc<str>, Arc<str>>), CoreError> {
        let code_comment_step = CustomStep {
            name: "Code Comment".into(),
            system_template: r#"
You are a code-commenting bot for a highly knowledgeable audience. 
Think step by step with a logical reasoning and intellectual sense before you provide any response.
Provide concise explanations of the underlying algorithms and the reasoning behind key 
implementation choices.
Focus on **non-obvious** aspects and design decisions that enhance understanding of the 
code's logic and purpose at a high level. 
Avoid trivial or self-evident comments. Do not comment trivia. 
Clarify why and how certain decisions impact the outcome, and how this strategy 
leads to the correct or optimal solution in a broader context. If there are existing 
comments, leave them intact and improve on them below.
Return the commented code only.
Use technical jargon.
            "#
            .into(),
            user_template: "Add comments for the code:\n{{code}}".into(),
            input_param_names: vec!["code".into()],
            output_param_name: "commented_code".into(),
        };

        let write_article_step = CustomStep {
            name: Arc::from("Write Article"),
            system_template:  r#"
You are a diligent bot that creates concise professional leetcode articles for a highly 
knowledgeable audience.
Think step by step with a logical reasoning and intellectual sense before you provide any response.
Outline key properties of input data and output data that unlock the solution.
You write articles so good that even a five-year-old will understand it.
You are focusing on the core concepts and ideas, completely ignoring trivia.
You explain concepts in a simple way, focus on ideas, gotchas, key concepts, rather implementations.
If the explanation would benefit from an example, you add it.
For a given code you create a nice idea-focused article, adding multiple steps if necessary to achieve the goal.
        "#.into(),
            user_template: "Write an article for the code below:\n{{code}}".into(),
            input_param_names: vec![Arc::from("commented_code")],
            output_param_name: Arc::from("article"),
        };

        let create_flashcards_step = CustomStep {
            name: Arc::from("Create Flashcards"),
            system_template: Arc::from(r#"
You a bot converting given leetcode code and article into flashcards.
Think step by step with a logical reasoning and intellectual sense before you provide any response.
You provide flashcards with a necessary amount of hints that would point to the right direction
without exposing solution completely, but you can share key insights progressively: the next card
can expose more details than the previous.
The amount of hints depends on the complexity of a problem.
It must be enough to remember the solution.
The given problem can have more than one flashcard if it is necessary. Don't create more than 3 cards.
You respond in this JSON format ONLY:

{
    importance: <rate the difficulty of the problem on scale 1-10>,  
    difficulty: <rate the importance of the problem in terms of its popularity and popularity of the concepts used in the solution on FAANG interviews>,
    title: <The problem title>,
    tags: [<create a list of tags for the problem and solution>],
    data: {
        leetcode_link: "https://...",
    },
    cards: [
        {
            title: <card title>, 
            front: <card front>, 
            back: <card back>,
            hints: [<list of hints>], 
            difficulty: <rate the difficulty of the card problem on scale 1-10>,  
            importance: <rate the importance of the problem in terms of its popularity and popularity of the concepts used in the solution on FAANG interviews>,
            tags: [<create a list of tags for the problem and solution>]
        },
        ...
    ]
}
"#),
            user_template: Arc::from("Convert given article and solution into flashcards:\nArticle:\n{{article}}\n\nCode:\n{{commented_code}}"),
            input_param_names: vec!["article".into(), "commented_code".into()],
            output_param_name: Arc::from("flashcards"),
        };

        let result = self
            .card_generator
            .execute_custom_chain(
                &[
                    code_comment_step,
                    write_article_step,
                    create_flashcards_step,
                ],
                code.as_ref(),
            )
            .await?;

        Ok(result)
    }

    pub async fn create_cards(
        &self,
        user: impl Into<Thing>,
        deck: impl Into<Thing>,
        gpt_card_group: GptCardGroup,
    ) -> Result<DeckCardGroupDto, CoreError> {
        let user = user.into();
        let deck = deck.into();

        let mut cards = vec![];
        for card in gpt_card_group.cards {
            let tags = self
                .tags
                .get_or_create_tags(user.clone(), card.tags)
                .await?;
            let card = self
                .cards
                .create(CreateCardDto {
                    user: user.clone(),
                    title: card.title,
                    front: Some(card.front),
                    back: Some(card.back),
                    hints: card.hints,
                    difficulty: card.difficulty,
                    importance: card.importance,
                    data: None,
                    tags: tags.into_iter().map(|t| t.id).collect(),
                })
                .await?;
            cards.push(card);
        }

        let tags = self
            .tags
            .get_or_create_tags(user.clone(), gpt_card_group.tags)
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect();

        let card_group = self
            .card_groups
            .create(CreateCardGroupDto {
                user: user.clone(),
                importance: gpt_card_group.importance,
                title: gpt_card_group.title,
                data: gpt_card_group.data.map(Arc::new),
                cards: cards.into_iter().map(|c| c.id).collect(),
                difficulty: gpt_card_group.difficulty,
                tags,
            })
            .await?;

        let deck_card_group = self
            .decks
            .relate_card_group(CreateDeckCardGroupDto {
                deck: deck.clone(),
                card_group: card_group.id,
            })
            .await?;

        Ok(deck_card_group)
    }
}
