use serde::Deserialize;

mod checks;
pub mod compare;

pub use checks::{last_checked_commit, last_run_id};

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub const BASE_URL: &str = "https://api.github.com/repos/bs-community/blessing-skin-server";

pub const WORKFLOW_ID: &u16 = &6505;

#[derive(Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetail,
}

#[derive(Deserialize)]
pub struct CommitDetail {
    pub message: String,
}
