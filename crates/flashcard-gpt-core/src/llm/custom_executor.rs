use crate::error::CoreError;
use itertools::Itertools;
use llm_chain::frame::Frame;
use llm_chain::step::Step;
use llm_chain::{prompt, Parameters};
use llm_chain_openai::chatgpt::Executor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
pub struct CustomExecutor {
    executor: Executor,
}

impl CustomExecutor {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
    pub async fn execute_custom_chain(
        &self,
        custom_steps: &[CustomStep],
        text: impl Into<String>,
    ) -> Result<(String, BTreeMap<Arc<str>, Arc<str>>), CoreError> {
        let mut parameters = Parameters::new();

        let Some(first_step) = custom_steps.first() else {
            return Err(CoreError::LlmNoLlmStepsProvided("No steps provided".into()));
        };
        if first_step.input_param_names.len() != 1 {
            let provided = Arc::from(first_step.input_param_names.join(", "));
            return Err(CoreError::LlmFirstStepInputParamError(provided));
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
                .cloned();

            let Some(result) = result else {
                return Err(CoreError::LlmBodyExtractError(
                    format!("Failed to extract body from step: {}", custom_step.name).into(),
                ));
            };

            info!(%custom_step.name, ?result, "Step executed");
            parameters = parameters.with(custom_step.output_param_name.as_ref(), result);
        }

        let Some(output_step) = custom_steps.last() else {
            return Err(CoreError::LlmNoLlmStepsProvided("No steps provided".into()));
        };

        let result = parameters.get(output_step.output_param_name.as_ref());
        let Some(result) = result else {
            return Err(CoreError::LlmResultMissing(
                format!(
                    "Output parameter not found: {}; exising: {parameters:?}",
                    output_step.output_param_name
                )
                .into(),
            ));
        };

        let return_params = custom_steps
            .iter()
            .flat_map(|step| {
                step.input_param_names
                    .iter()
                    .chain(std::iter::once(&step.output_param_name))
                    .cloned()
            })
            .unique()
            .map(|name| {
                let key = name.clone();
                let value = Arc::from(parameters.get(name.as_ref()).unwrap());
                (key, value)
            })
            .collect::<BTreeMap<Arc<str>, Arc<str>>>();

        Ok((result, return_params))
    }
}
