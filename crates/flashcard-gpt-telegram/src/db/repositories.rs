use flashcard_gpt_core::reexports;
use flashcard_gpt_core::reexports::db::engine::remote::ws::Client;
use flashcard_gpt_core::reexports::db::Surreal;
use flashcard_gpt_core::repo::binding::BindingRepo;
use flashcard_gpt_core::repo::card::CardRepo;
use flashcard_gpt_core::repo::user::UserRepo;

#[derive(Debug, Clone)]
pub struct Repositories {
    pub users: UserRepo,
    pub cards: CardRepo,
    pub bindings: BindingRepo,
}

impl Repositories {
    pub fn new(db: Surreal<Client>, span: reexports::trace::Span) -> Self {
        Self {
            users: UserRepo::new_user(db.clone(), span.clone(), true),
            cards: CardRepo::new_card(db.clone(), span.clone(), true),
            bindings: BindingRepo::new_binding(db, span, true),
        }
    }
}
