use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct Bot {
    url: String,
    chat_id: String,
}

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Network error related to `reqwest`.")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse JSON from Telegram response.")]
    JsonParsing(#[from] serde_json::Error),

    #[error("Telegram error: {0}")]
    Telegram(String),

    #[error("Telegram error: unknown")]
    TelegramUnknown,
}

#[derive(Deserialize)]
struct TelegramResponse {
    ok: bool,
    description: Option<String>,
}

#[derive(Serialize)]
struct SendMessage {
    chat_id: String,
    text: String,
    parse_mode: &'static str,
    disable_notification: bool,
}

impl Bot {
    pub fn new(token: &str, chat_id: &str) -> Self {
        Self {
            url: format!("https://api.telegram.org/bot{}/sendMessage", token),
            chat_id: chat_id.into(),
        }
    }

    pub async fn send_message<S, M>(&self, text: S, parse_mode: M) -> Result<(), BotError>
    where
        S: Into<String>,
        M: Into<Option<&'static str>>,
    {
        let client = reqwest::Client::new();
        let TelegramResponse { ok, description } = client
            .post(&self.url)
            .json(&SendMessage {
                chat_id: self.chat_id.clone(),
                text: text.into(),
                parse_mode: parse_mode.into().unwrap_or("Markdown"),
                disable_notification: true,
            })
            .send()
            .await?
            .json::<TelegramResponse>()
            .await?;

        if ok {
            Ok(())
        } else {
            match description {
                Some(description) => Err(BotError::Telegram(description)),
                None => Err(BotError::TelegramUnknown),
            }
        }
    }
}
