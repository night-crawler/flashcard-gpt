use anyhow::{bail, Context};
use llm_chain::frame::Frame;
use llm_chain::step::Step;
use llm_chain::{prompt, Parameters};
use llm_chain_openai::chatgpt::Executor;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomStep {
    pub name: Arc<str>,
    pub system_template: Arc<str>,
    pub user_template: Arc<str>,
    pub input_param_names: Vec<Arc<str>>,
    pub output_param_name: Arc<str>,
}

impl CustomStep {
    pub fn to_step(&self) -> Step {
        let prompt = prompt!(self.system_template.as_ref(), self.user_template.as_ref());
        Step::for_prompt_template(prompt)
    }
}

#[derive(Clone)]
pub struct CardGenerationService {
    executor: Executor,
}

impl CardGenerationService {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
    pub async fn execute_custom_chain(
        &self,
        custom_steps: &[CustomStep],
        text: impl Into<String>,
    ) -> anyhow::Result<String> {
        let mut parameters = Parameters::new();

        let first_step = custom_steps.first().with_context(|| "No steps provided")?;
        if first_step.input_param_names.len() != 1 {
            bail!("First step must have exactly one input parameter");
        }

        parameters = parameters.with(
            first_step
                .input_param_names
                .first()
                .as_ref()
                .unwrap()
                .as_ref(),
            text,
        );

        for custom_step in custom_steps {
            info!(%custom_step.name, "Executing step");
            let step = custom_step.to_step();
            let result = Frame::new(&self.executor, &step)
                .format_and_execute(&parameters)
                .await?
                .to_immediate()
                .await?
                .as_content()
                .extract_last_body()
                .with_context(|| {
                    format!("Failed to extract last body for step: {:?}", custom_step)
                })?
                .clone();
            info!(%custom_step.name, ?result, "Step executed");
            parameters = parameters.with(custom_step.output_param_name.as_ref(), result);
        }

        let output_step = custom_steps.last().with_context(|| "No steps provided")?;
        let result = parameters
            .get(output_step.output_param_name.as_ref())
            .with_context(|| {
                format!(
                    "Output parameter not found: {}",
                    output_step.output_param_name
                )
            })?;

        Ok(result)
    }

    pub async fn generate_code_card(&self, code: impl AsRef<str>) -> anyhow::Result<String> {
        let code_comment_step = CustomStep {
            name: "Code Comment".into(),
            system_template: r#"
You are a code-commenting bot for a highly knowledgeable audience. 
Provide concise explanations of the underlying algorithms and the reasoning behind key 
implementation choices. 
Focus on **non-obvious** aspects and design decisions that enhance understanding of the 
code's logic and purpose at a high level. 
Avoid trivial or self-evident comments. Do not comment trivia. 
Clarify why certain concatenation decisions impact the outcome, and how this strategy 
leads to the correct or optimal solution in a broader context. If there are existing 
comments, leave them intact and improve on them below.
Return the commented code only.
Do not avoid technical jargon.
            "#
            .into(),
            user_template: "Add comments for the code:\n{{code}}".into(),
            input_param_names: vec!["code".into()],
            output_param_name: "commented_code".into(),
        };

        let write_article_step = CustomStep {
            name: Arc::from("Write Article"),
            system_template:  r#"
You are a diligent bot that creates leetcode articles.
You write articles so good that even a five-year-old will understand it.
You are not focusing on trivia, you are focusing on the core concepts, ideas rather than implementation.
You explain concepts in a simple way, focus on ideas, gotchas, key concepts, rather implementations.
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
You provide flashcards with a necessary amount of hints that would point to the right direction without exposing solution. 
The amount of hints depends on complexity of a problem. It must be enough to remember the solution. 
The given problem can have more than one flashcard if it is necessary. Don't create more than 3 cards.
You respond in this JSON format ONLY:

{
    importance: <rate the difficulty of the card group on scale 1-10>,  
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
            difficulty: <rate the difficulty of the card on scale 1-10>,  
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use flashcard_gpt_core::logging::init_tracing;
    use llm_chain::options::{ModelRef, Opt, Options};
    use llm_chain::traits::Executor;
    use testresult::TestResult;

    #[tokio::test]
    async fn test_generate_card() -> TestResult {
        init_tracing()?;
        //  api key from env
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let mut options = Options::builder();
        options.add_option(Opt::ApiKey(api_key));
        options.add_option(Opt::Model(ModelRef::from_model_name("chatgpt-4o-latest")));
        let options = options.build();

        let exec = llm_chain_openai::chatgpt::Executor::new_with_options(options)?;

        let service = CardGenerationService::new(exec);
        let code = r#"
        
use std::cmp::Ordering;

/// https://leetcode.com/problems/largest-number/
pub struct Solution;
impl Solution {
    pub fn largest_number(nums: Vec<i32>) -> String {
        let mut nums = nums.into_iter().map(|num| num.to_string()).collect::<Vec<_>>();
        nums.sort_unstable_by(|a, b| {
            let ab = format!("{a}{b}");
            let ba = format!("{b}{a}");
            if ab.parse::<usize>().unwrap() > ba.parse::<usize>().unwrap() {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        if nums[0].starts_with('0') {
            return "0".to_string();
        }

        nums.join("")
    }
}
        "#;
        service.generate_code_card(code).await?;

        Ok(())
    }
}
