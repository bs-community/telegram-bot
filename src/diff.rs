use crate::bot::{Bot, BotError};
use std::process::Command;
use thiserror::Error;
use tokio::fs;

pub async fn execute(bot: &Bot) -> Result<(), DiffError> {
    let (head, diff) = futures::join!(head(), diff());
    let head = head?;
    let diff = diff.map(analyze_diff)?;

    bot.send_message(head + "\n" + &diff)
        .await
        .map_err(DiffError::from)
}

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("IO error.")]
    IO(#[from] std::io::Error),

    #[error("Failed to collect stdout.")]
    Stdout(#[from] std::string::FromUtf8Error),

    #[error("Bot error.")]
    Bot(#[from] BotError),
}

#[derive(Clone)]
struct Diff {
    build: bool,
    yarn: bool,
    composer: bool,
}

async fn diff() -> Result<Diff, DiffError> {
    Command::new("git")
        .args(&["difftool", "--output=diff.txt", "--name-only", "HEAD^"])
        .output()?;

    let mut diff = Diff {
        build: false,
        yarn: false,
        composer: false,
    };

    let diff_text = String::from_utf8(fs::read("diff.txt").await?)?;
    for change in diff_text.lines() {
        match change {
            "package.json" | "yarn.lock" => {
                diff.yarn = true;
            }
            "composer.json" | "composer.lock" => {
                diff.composer = true;
            }
            change => {
                if change.starts_with("resources/assets/src") {
                    diff.build = true;
                }
            }
        }
    }

    Ok(diff)
}

fn analyze_diff(
    Diff {
        build,
        yarn,
        composer,
    }: Diff,
) -> String {
    if !build && !yarn && !composer {
        return "您可以直接拉取此 commit。".to_string();
    }

    let front: &'static str;
    let back: &'static str;

    if yarn {
        front = "先执行 `yarn`，然后执行 `pwsh ./scripts/build.ps1`。";
    } else if build {
        front = "执行 `pwsh ./scripts/build.ps1`。";
    } else {
        front = "";
    }

    if composer {
        back = "执行 `composer install`。";
    } else {
        back = "";
    }

    format!("拉取此 commit 后，您需要：\n{}\n{}", front, back)
}

async fn head() -> Result<String, DiffError> {
    let command = Command::new("git")
        .args(&["log", "--pretty=*%h*: %s", "-1"])
        .output()?;

    String::from_utf8(command.stdout).map_err(DiffError::from)
}

#[test]
fn test_analyze_diff() {
    let mut diff = Diff {
        build: false,
        yarn: false,
        composer: false,
    };

    assert_eq!(&analyze_diff(diff.clone()), "您可以直接拉取此 commit。");

    diff.yarn = true;
    let analysis = analyze_diff(diff.clone());
    assert!(analysis.contains("`yarn`"));
    assert!(analysis.contains("pwsh"));

    diff.yarn = false;
    diff.build = true;
    let analysis = analyze_diff(diff.clone());
    assert!(!analysis.contains("`yarn`"));
    assert!(analysis.contains("pwsh"));

    diff.composer = true;
    let analysis = analyze_diff(diff.clone());
    assert!(analysis.contains("`composer install`"));
}

#[test]
fn test_head() {
    use tokio::runtime::current_thread::Runtime;

    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        let output = head().await.unwrap();
        assert!(output.ends_with('\n'));

        let parts = output.split(':').collect::<Vec<&str>>();
        let left = parts[0];
        let right = parts[1];

        assert!(left.starts_with('*'));
        assert!(left.ends_with('*'));
        assert_eq!(left.len(), 9);

        assert!(right.starts_with(' '));
    });
}
