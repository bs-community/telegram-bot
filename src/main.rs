use std::env;
use thiserror::Error;

mod bot;
mod diff;
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
    let mut args = env::args();
    args.next().unwrap();
    let action = &*args.next().expect("Missing action name");

    let mut chat_id = env::var("TELEGRAM_CHAT_ID").expect("Missing Telegram chat ID");
    if !chat_id.starts_with("@") {
        chat_id = String::from("@") + &chat_id;
    }
    let bot = bot::Bot::new(
        &env::var("TELEGRAM_BOT_TOKEN").expect("Missing Telegram bot token"),
        &chat_id,
    );

    match action {
        "plugin" => {
            let path = args.next().expect("Missing JSON file path.");
            plugin::execute(&bot, path).await.map_err(MainError::from)
        }
        "diff" | "core" => diff::execute(&bot).await.map_err(MainError::from),
        unknown => Err(MainError::UnknownAction(unknown.into())),
    }
}
