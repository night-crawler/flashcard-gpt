use crate::command::CommandExt;
use crate::FlashGptDialogue;
use std::future::Future;

pub trait DialogueExt {
    fn set_menu_state<T>(&self) -> impl Future<Output = anyhow::Result<()>>
    where
        T: CommandExt;
}

impl DialogueExt for FlashGptDialogue {
    async fn set_menu_state<T>(&self) -> anyhow::Result<()>
    where
        T: CommandExt,
    {
        self.update(T::get_corresponding_state()).await?;
        Ok(())
    }
}
