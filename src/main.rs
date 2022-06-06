#[macro_use]
extern crate log;

use env_logger::Env;
use std::env;

mod bot;
mod diff;
mod github;
mod plugin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("bot=debug")).init();

    info!("BS Telegram Bot: v{}", env!("CARGO_PKG_VERSION"));

    let mut args = env::args();
    args.next().unwrap();
    let action = &*args.next().expect("Missing action name");

    let bot = bot::Bot::new(
        &env::var("TELEGRAM_BOT_TOKEN")?,
        &env::var("TELEGRAM_CHAT_ID")?,
    );

    info!("Current action is {}", action);
    match action {
        "plugin" => {
            let path = args.next().expect("Missing JSON file path.");
            plugin::execute(&bot, path).await.map_err(anyhow::Error::from)
        }
        "diff" | "core" => {
            let base = args.next();
            let head = args.next();
            diff::execute(&bot, base, head)
                .await.map_err(anyhow::Error::from)
        }
        unknown => Err(anyhow::Error::msg(format!("Unknown action: {}.", unknown))),
    }
}
