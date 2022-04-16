#[macro_use]
extern crate log;

use env_logger::Env;
use std::env;
use thiserror::Error;

mod bot;
mod diff;
mod github;
mod plugin;

#[derive(Error, Debug)]
enum MainError {
    #[error("[Plugin] {0}")]
    Plugin(#[from] plugin::PluginDataError),

    #[error("[Diff] {0}")]
    Diff(#[from] diff::DiffError),

    #[error("Unknown action name: '{0}'.")]
    UnknownAction(String),
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    env_logger::from_env(Env::default().default_filter_or("bot=debug")).init();

    info!("BS Telegram Bot: v{}", env!("CARGO_PKG_VERSION"));

    let mut args = env::args();
    args.next().unwrap();
    let action = &*args.next().expect("Missing action name");

    let bot = bot::Bot::new(
        &env::var("TELEGRAM_BOT_TOKEN").expect("Missing Telegram bot token"),
        &env::var("TELEGRAM_CHAT_ID").expect("Missing Telegram chat ID"),
    );

    info!("Current action is {}", action);
    match action {
        "plugin" => {
            let path = args.next().expect("Missing JSON file path.");
            plugin::execute(&bot, path).await.map_err(MainError::from)
        }
        "diff" | "core" => {
            let base = args.next();
            let head = args.next();
            let run_id = env::var("GITHUB_RUN_ID").ok();
            diff::execute(&bot, base, head, run_id)
                .await
                .map_err(MainError::from)
        }
        unknown => Err(MainError::UnknownAction(unknown.into())),
    }
}
