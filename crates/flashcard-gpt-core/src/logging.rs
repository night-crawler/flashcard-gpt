use crate::error::CoreError;
use console_subscriber::{ConsoleLayer, Server, ServerAddr};
use std::net::SocketAddr;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() -> Result<(), CoreError> {
    let addr = ServerAddr::Tcp(SocketAddr::new(Server::DEFAULT_IP, 6660));
    let console_layer = ConsoleLayer::builder()
        .with_default_env()
        .server_addr(addr)
        .spawn();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(console_layer)
        .try_init()?;

    Ok(())
}
