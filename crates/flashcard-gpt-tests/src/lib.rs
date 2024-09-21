use ctor::ctor;
use flashcard_gpt_core::logging::init_tracing;

pub mod db;

#[ctor]
fn initialize_tracing() {
    if let Err(e) = init_tracing() {
        eprintln!("Error initializing tracking: {e:?}");
    }
}
