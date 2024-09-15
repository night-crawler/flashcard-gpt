

#[macro_export]
macro_rules! patch_state {
    ($manager:expr, $current_state:path { $($field:ident),* $(,)? }, $mutation:expr) => {
        paste::item! {
            {
                let mut fields = $manager.get_state().await?.take_fields(); 
                
                if let $current_state { $( [< $field >]),* , .. } = &mut fields {
                    ($mutation)($([< $field >]),*);
                } else {
                    anyhow::bail!("Unexpected state: {:?}", fields);
                }
                
                fields
            }
        }
    };
}

