use super::{BASE_URL, USER_AGENT};
use serde::Deserialize;

#[derive(Deserialize)]
struct CheckSuite {
    before: String,
}

#[derive(Deserialize)]
struct CheckSuites {
    total_count: u32,
    check_suites: Vec<CheckSuite>,
}

pub async fn last_checked_commit() -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?;
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

#[derive(Deserialize)]
struct WorkflowRun {
    id: u64,
    workflow_id: u16,
}

#[derive(Deserialize)]
struct Runs {
    total_count: u32,
    workflow_runs: Vec<WorkflowRun>,
}

pub async fn last_run_id(id: &u16, branch: &str) -> Result<Option<u64>, reqwest::Error> {
    let client = reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?;

    let Runs {
        total_count,
        workflow_runs,
    } = client
        .get(&format!("{}/actions/runs?branch={}", BASE_URL, branch))
        .header("Accept", "application/vnd.github.antiope-preview+json")
        .send()
        .await?
        .json()
        .await?;

    let run_id = if total_count == 0 {
        None
    } else {
        workflow_runs
            .into_iter()
            .find(|run| &run.workflow_id == id)
            .map(|run| run.id)
    };
    Ok(run_id)
}
