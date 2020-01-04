use super::{Commit, BASE_URL};
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
    let client = reqwest::ClientBuilder::new().gzip(true).build()?;

    client.get(&url).send().await?.json().await
}
