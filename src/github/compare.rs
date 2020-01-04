use super::{Commit, BASE_URL, USER_AGENT};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct File {
    pub filename: String,
}

#[derive(Deserialize)]
pub struct Compare {
    pub commits: Vec<Commit>,
    pub files: Vec<File>,
}

pub async fn compare(base: &str, head: &str) -> Result<Compare, reqwest::Error> {
    let url = format!("{}/compare/{}...{}", BASE_URL, base, head);
    let client = reqwest::ClientBuilder::new()
        .gzip(true)
        .user_agent(USER_AGENT)
        .build()?;

    client.get(&url).send().await?.json().await
}
