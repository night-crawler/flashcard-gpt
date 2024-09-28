use flashcard_gpt_core::llm::card_generator::CardGenerator;
use std::sync::Arc;

use flashcard_gpt_core::dto::llm::{GptCard, GptCardGroup};
use flashcard_gpt_core::llm::card_generator_service::CardGeneratorService;
use flashcard_gpt_tests::db::utils::{
    create_card_group_repo, create_card_repo, create_deck, create_deck_repo, create_tag,
    create_tag_repo, create_user,
};
use llm_chain::options::{ModelRef, Opt, Options};
use llm_chain::traits::Executor;
use serde_json::json;
use testresult::TestResult;
use tracing::error;

#[tokio::test]
async fn test_generate_card() -> TestResult {
    let Ok(api_key) = std::env::var("OPENAI_API_KEY") else {
        error!("OPENAI_API_KEY not set");
        return Ok(());
    };
    let mut options = Options::builder();
    options.add_option(Opt::ApiKey(api_key));
    options.add_option(Opt::Model(ModelRef::from_model_name("chatgpt-4o-latest")));
    let options = options.build();

    let exec = llm_chain_openai::chatgpt::Executor::new_with_options(options)?;
    let generator = CardGenerator::new(exec);

    let card_generator_service = CardGeneratorService {
        card_generator: generator,
        cards: create_card_repo().await?,
        card_groups: create_card_group_repo().await?,
        decks: create_deck_repo().await?,
        tags: create_tag_repo().await?,
    };

    let code = include_str!("./sample_code.txt");
    card_generator_service.generate_code_cards(code).await?;

    Ok(())
}

#[tokio::test]
async fn test_create_cards() -> TestResult {
    let user = create_user("test_create_cards").await?;
    let tag = create_tag()
        .user(&user)
        .name("test_create_cards")
        .slug("test_create_cards")
        .call()
        .await?;
    let deck = create_deck()
        .user(&user)
        .title("test_create_cards")
        .tags([&tag])
        .call()
        .await?;

    let gpt_card_group = GptCardGroup {
        importance: 10,
        difficulty: 2,
        title: Arc::from("title"),
        tags: vec![Arc::from("tag1"), Arc::from("tag2"), Arc::from("tag3")],
        data: Some(json!({"leetcode_link": "https://leetcode.com"})),
        cards: vec![
            GptCard {
                title: Arc::from("title1"),
                front: Arc::from("front1"),
                back: Arc::from("back1"),
                hints: vec![Arc::from("hint1")],
                difficulty: 2,
                importance: 2,
                tags: vec![Arc::from("tag1"), Arc::from("tag one"), Arc::from("tag 1")],
            },
            GptCard {
                title: Arc::from("title2"),
                front: Arc::from("front2"),
                back: Arc::from("back2"),
                hints: vec![Arc::from("hint2")],
                difficulty: 3,
                importance: 3,
                tags: vec![Arc::from("tag2"), Arc::from("tag two"), Arc::from("tag 2")],
            },
        ],
    };

    let card_generator_service = CardGeneratorService {
        card_generator: CardGenerator::new(llm_chain_openai::chatgpt::Executor::new_with_options(
            Options::default(),
        )?),
        cards: create_card_repo().await?,
        card_groups: create_card_group_repo().await?,
        decks: create_deck_repo().await?,
        tags: create_tag_repo().await?,
    };

    let deck_card_group = card_generator_service
        .create_cards(user.id, deck.id, gpt_card_group)
        .await?;

    assert_eq!(deck_card_group.card_group.cards.len(), 2);

    Ok(())
}
