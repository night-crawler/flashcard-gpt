use crate::model::card::CreateCard;
use crate::model::card_group::CreateCardGroup;
use crate::model::deck_card_group::{CreateDeckCardGroup, DeckCardGroup};
use crate::model::llm::GptCardGroup;
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
Provide concise explanations of the underlying algorithms and the reasoning behind key 
implementation choices.
Focus on **non-obvious** aspects and design decisions that enhance understanding of the 
code's logic and purpose at a high level. 
Avoid trivial or self-evident comments.
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
            system_template: r#"
You are a diligent bot that creates concise, professional LeetCode articles for a highly 
knowledgeable audience.
Focus on core concepts, key ideas, and potential pitfalls, avoiding trivial details.
Explain the underlying logic and reasoning behind solutions, emphasizing how the properties of input
and output data influence the approach.
Provide clear explanations without oversimplifying, ensuring depth and precision.
Include examples when they enhance understanding of complex concepts.
For a given code, create an idea-focused article that thoroughly explains the solution, adding 
multiple steps if necessary to achieve clarity.
        "#
            .into(),
            user_template: "Write an article for the code below:\n{{code}}".into(),
            input_param_names: vec![Arc::from("commented_code")],
            output_param_name: Arc::from("article"),
        };

        let create_flashcards_step = CustomStep {
            name: Arc::from("Create Flashcards"),
            system_template: Arc::from(r#"
You are a bot that converts given LeetCode code and articles into flashcards.
For each problem, create flashcards that include hints pointing in the right direction without fully
exposing the solution.
Share key insights progressively, with each subsequent card revealing more details than the previous
one to guide the user toward the solution.
The number of hints and cards should correspond to the complexity of the problem, but do not create 
more than **3 cards per problem**.
Ensure that the hints are sufficient for the user to recall the solution.
Respond **only** in the following JSON format (do not include any additional text outside the JSON):

{
 "title": "<The problem title>",
 "difficulty": <rate the difficulty of the problem on a scale of 1-10 (1 = easiest, 10 = hardest)>,
 "importance": <rate the importance of the problem on a scale of 1-10 (1 = least important, 10 = most important), based on its popularity and the frequency of the concepts used in FAANG interviews>,
 "tags": [<list of relevant tags for the problem and solution, e.g., 'Dynamic Programming', 'Graphs', 'Recursion'>],
 "data": {
   "leetcode_link": "https://..."
 },
 "cards": [
   {
     "title": "<Card title>",
     "front": "<Card front text>",
     "back": "<Card back text>",
     "hints": ["<List of hints>"],
     "difficulty": <rate the difficulty of the card on a scale of 1-10 (1 = easiest, 10 = hardest)>,
     "importance": <rate the importance of the card on a scale of 1-10 (1 = least important, 10 = most important), based on the concepts it covers>,
     "tags": ["<List of relevant tags for the card>"]
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
    ) -> Result<DeckCardGroup, CoreError> {
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
                .create(CreateCard {
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
            .create(CreateCardGroup {
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
            .relate_card_group(CreateDeckCardGroup {
                deck: deck.clone(),
                card_group: card_group.id,
            })
            .await?;

        Ok(deck_card_group)
    }
}
