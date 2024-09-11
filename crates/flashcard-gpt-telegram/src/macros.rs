#[macro_export]
macro_rules! propagate {
    ($dialogue:expr, $current_state:path { $($cur_field:ident),* $(,)? }, $next_state:path { $($next_field:ident: $next_value:expr),* }) => {
        paste::item! {
            {
                let state = $dialogue.get_or_default().await?;
                if let $current_state { $([< $cur_field >]),* } = state {
                    $dialogue.update($next_state {
                        $([< $cur_field >]),*,
                        $([< $next_field >]: $next_value),*
                    }).await?;
                    $dialogue.get_or_default().await?
                } else {
                    anyhow::bail!("Unexpected state: {:?}", state);
                }
            }
        }
    };
}


#[macro_export]
macro_rules! patch_state {
    // The base case: takes a dialogue, current state pattern, and a mutation function that operates on the fields
    ($dialogue:expr, $current_state:path { $($field:ident),* $(,)? }, $mutation:expr) => {
        paste::item! {
            {
                // Get the current state from the dialogue
                let mut state = $dialogue.get_or_default().await?;
                // Check if the state matches the provided pattern
                if let $current_state { $(ref mut [< $field >]),* , .. } = &mut state {
                    // Apply the mutation closure
                    ($mutation)($([< $field >]),*);
                    // Update the dialogue with the mutated state
                    $dialogue.update(state).await?;
                    // Return the updated state
                    $dialogue.get_or_default().await?
                } else {
                    // Handle the case where the state doesn't match the expected pattern
                    anyhow::bail!("Unexpected state: {:?}", state);
                }
            }
        }
    };
}
