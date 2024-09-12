use crate::command::CommandExt;
use crate::FlashGptDialogue;
use std::future::Future;
use teloxide::types::Message;
use crate::state::StateDescription;

pub trait DialogueExt {
    fn set_menu_state<T>(&self) -> impl Future<Output = anyhow::Result<()>>
    where
        T: CommandExt;
    
    fn get_state_description(&self, msg: Option<&Message>) -> impl Future<Output=anyhow::Result<StateDescription>>;
}

impl DialogueExt for FlashGptDialogue {
    async fn set_menu_state<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        self.update(T::get_corresponding_state()).await?;
        Ok(())
    }

    async fn get_state_description(&self, msg: Option<&Message>) -> anyhow::Result<StateDescription> {
        let Some(state) = self.get().await? else {
            return Ok(StateDescription::default());
        };
        
        Ok(state.get_state_description(msg))
    }
}
