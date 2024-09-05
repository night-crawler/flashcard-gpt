#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    InsideRootMenu,
    InsideUserMenu,
    InsideDeckMenu,
    InsideCardMenu,
    InsideCardGroupMenu,

    ReceiveFullName,
    ReceiveProductChoice {
        full_name: String,
    },
}
