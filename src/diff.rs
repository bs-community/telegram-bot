use crate::bot::{Bot, BotError};
use std::process::Command;
use thiserror::Error;
use tokio::fs;

pub async fn execute(bot: &Bot) -> Result<(), DiffError> {
    let (head, diff, hitokoto) = futures::join!(head(), diff(), hitokoto());
    let head = head?;
    let diff = diff.map(analyze_diff)?;
    let hitokoto = hitokoto.unwrap_or(String::new());

    let message = head + "\n" + &diff + &hitokoto;
    bot.send_message(message, "HTML")
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
        use rand::seq::IteratorRandom;

        let mut rng = rand::thread_rng();
        let choices = vec![
            "您可以直接拉取此 commit。",
            "赶快拿起 <code>git pull</code>订购吧！",
            "是朋友，就 <code>git pull</code>。介是你没有用过的船新版本。",
        ];
        return choices.into_iter().choose(&mut rng).unwrap().to_string();
    }

    let front: &'static str;
    let back: &'static str;

    if yarn {
        front = "先执行 <code>yarn</code>，然后执行 <code>pwsh ./scripts/build.ps1</code>。";
    } else if build {
        front = "执行 <code>pwsh ./scripts/build.ps1</code>。";
    } else {
        front = "";
    }

    if composer {
        back = "执行 <code>composer install</code>。";
    } else {
        back = "";
    }

    format!("拉取此 commit 后，您需要：\n{}\n{}", front, back)
}

async fn head() -> Result<String, DiffError> {
    let command = Command::new("git")
        .args(&["log", "--pretty=**%h**: %s", "-1"])
        .output()?;

    String::from_utf8(command.stdout)
        .map(md2html)
        .map_err(DiffError::from)
}

async fn hitokoto() -> Result<String, reqwest::Error> {
    reqwest::get("https://v1.hitokoto.cn/?encode=text")
        .await?
        .text()
        .await
        .map(|text| format!("\n---\n{}", text))
}

fn md2html(text: String) -> String {
    use pulldown_cmark::{html, Parser};

    let text = text.replace("<", "&lt;").replace(">", "&gt;");
    let parser = Parser::new(&text);
    let mut output = String::new();
    html::push_html(&mut output, parser);

    output
        .trim()
        .trim_start_matches("<p>")
        .trim_end_matches("</p>")
        .into()
}

#[test]
fn test_analyze_diff() {
    let mut diff = Diff {
        build: false,
        yarn: false,
        composer: false,
    };

    let analysis = analyze_diff(diff.clone());
    assert!(!analysis.contains("yarn"));
    assert!(!analysis.contains("pwsh"));
    assert!(!analysis.contains("composer"));

    diff.yarn = true;
    let analysis = analyze_diff(diff.clone());
    assert!(analysis.contains("yarn"));
    assert!(analysis.contains("pwsh"));

    diff.yarn = false;
    diff.build = true;
    let analysis = analyze_diff(diff.clone());
    assert!(!analysis.contains("yarn"));
    assert!(analysis.contains("pwsh"));

    diff.composer = true;
    let analysis = analyze_diff(diff.clone());
    assert!(analysis.contains("composer install"));
}

#[test]
fn test_head() {
    use tokio::runtime::current_thread::Runtime;

    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        let output = head().await.unwrap();
        assert!(!output.contains("<p>"));
        assert!(!output.contains("</p>"));

        let parts = output.split(':').collect::<Vec<&str>>();
        let left = parts[0];
        let right = parts[1];

        assert!(left.starts_with("<strong>"));
        assert!(left.ends_with("</strong>"));
        assert_eq!(left.len(), 24);

        assert!(right.starts_with(' '));
    });
}

#[test]
fn test_md2html() {
    assert_eq!("&lt;modal&gt;", &md2html(String::from("<modal>")));
    assert_eq!("&quot;text&quot;", &md2html(String::from("\"text\"")));
    assert_eq!("&amp;", &md2html(String::from("&")));
    assert_eq!("<strong>bold</strong>", &md2html(String::from("**bold**")));
}
