use super::BASE_URL;
use serde::Deserialize;

#[derive(Deserialize)]
struct CheckSuite {
    before: String,
}

#[derive(Deserialize)]
struct CheckSuites {
    total_count: u8,
    check_suites: Vec<CheckSuite>,
}

pub async fn last_checked_commit() -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::ClientBuilder::new().gzip(true).build()?;
    let CheckSuites {
        total_count,
        check_suites,
    } = client
        .get(&format!("{}/commits/dev/check-suites", BASE_URL))
        .header("Accept", "application/vnd.github.antiope-preview+json")
        .send()
        .await?
        .json()
        .await?;

    let sha = if total_count == 0 {
        None
    } else {
        check_suites
            .last()
            .map(|check_suite| check_suite.before.clone())
    };
    Ok(sha)
}
