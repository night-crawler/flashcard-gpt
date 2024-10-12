pub mod answer;
pub mod card;
pub mod card_group;
pub mod deck;
pub mod ext;
pub mod root;
pub mod tag;
pub mod user;

pub use answer::*;
pub use card::*;
pub use card_group::*;
pub use deck::*;
use itertools::Itertools;
pub use root::*;
pub use tag::*;
use teloxide::types::BotCommand;
use teloxide::utils::command::BotCommands;
pub use user::*;

pub fn all_commands() -> impl Iterator<Item = BotCommand> {
    AnswerCommand::bot_commands()
        .into_iter()
        .chain(CardCommand::bot_commands())
        .chain(CardGroupCommand::bot_commands())
        .chain(DeckCommand::bot_commands())
        .chain(RootCommand::bot_commands())
        .chain(TagCommand::bot_commands())
        .chain(UserCommand::bot_commands())
        .sorted_unstable_by(|cmd1, cmd2| cmd1.command.cmp(&cmd2.command))
        .dedup_by(|cmd1, cmd2| cmd1.command == cmd2.command)
}
