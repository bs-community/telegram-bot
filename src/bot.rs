use serde::{Deserialize, Serialize};

pub struct Bot {
    url: String,
    chat_id: String,
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
        let chat_id = if chat_id.starts_with('@') {
            chat_id.into()
        } else {
            String::from("@") + chat_id
        };

        Self {
            url: format!("https://api.telegram.org/bot{}/sendMessage", token),
            chat_id,
        }
    }

    pub async fn send_message<S, M>(&self, text: S, parse_mode: M) -> anyhow::Result<()>
    where
        S: Into<String>,
        M: Into<Option<&'static str>>,
    {
        let text = text.into();
        let parse_mode = parse_mode.into().unwrap_or("Markdown");

        debug!(
            "Content sent to Telegram (parse mode is {}):\n{}",
            parse_mode, text
        );

        let client = reqwest::ClientBuilder::new().build()?;
        let TelegramResponse { ok, description } = client
            .post(&self.url)
            .json(&SendMessage {
                chat_id: self.chat_id.clone(),
                text,
                parse_mode,
                disable_notification: true,
            })
            .send()
            .await?
            .json()
            .await?;

        if ok {
            info!("Message sent successfully.");
            Ok(())
        } else {
            match description {
                Some(description) => {
                    Err(anyhow::Error::msg(format!("Telegram reported an error: {}", description)))
                }
                None => {
                    Err(anyhow::Error::msg("An unknown Telegram error occurred."))
                }
            }
        }
    }
}
