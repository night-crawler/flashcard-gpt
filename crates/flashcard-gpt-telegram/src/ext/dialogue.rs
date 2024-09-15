use crate::state::StateDescription;
use crate::FlashGptDialogue;
use std::future::Future;
use teloxide::types::Message;

pub trait DialogueExt {
    
    fn get_state_description(
        &self,
        msg: Option<&Message>,
    ) -> impl Future<Output = anyhow::Result<StateDescription>>;
}

impl DialogueExt for FlashGptDialogue {
   

    async fn get_state_description(
        &self,
        msg: Option<&Message>,
    ) -> anyhow::Result<StateDescription> {
        let Some(state) = self.get().await? else {
            return Ok(StateDescription::default());
        };

        Ok(state.get_state_description(msg))
    }
}
