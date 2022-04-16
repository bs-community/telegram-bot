use crate::bot::{Bot, BotError};
use crate::github::{
    self,
    compare::{compare, Compare},
};
use itertools::Itertools;
use thiserror::Error;

pub async fn execute(
    bot: &Bot,
    base: Option<String>,
    head: Option<String>,
    run_id: Option<String>,
) -> Result<(), DiffError> {
    let git = git(base, head, run_id).await?;
    bot.send_message(git, "HTML").await.map_err(DiffError::from)
}

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Bot error.")]
    Bot(#[from] BotError),
}

async fn git(
    base: Option<String>,
    head: Option<String>,
    run_id: Option<String>,
) -> Result<String, reqwest::Error> {
    let base = if let Some(base) = base {
        base
    } else {
        github::last_checked_commit()
            .await?
            .unwrap_or_else(|| String::from("HEAD^"))
    };
    let head = head.unwrap_or_else(|| "HEAD".into());

    info!("Base ref is {}", base);
    info!("Head ref is {}", head);

    let Compare { commits, files } = compare(&base, &head).await?;
    let log = format_log(&commits);
    let analysis = analyze_diff(diff(&files));
    let artifact = get_artifact_link(run_id);

    Ok(format!("{}\n{}\n{}", log, analysis, artifact))
}

#[derive(Clone, Default)]
struct Diff {
    build: bool,
    yarn: bool,
    composer: bool,
    migration: bool,
}

fn diff(files: &[github::compare::File]) -> Diff {
    let mut diff = Diff::default();

    for file in files {
        match &*file.filename {
            "package.json" | "yarn.lock" => {
                diff.yarn = true;
                diff.build = true;
            }
            "composer.json" | "composer.lock" => {
                diff.composer = true;
            }
            "webpack.config.js" => {
                diff.build = true;
            }
            change => {
                if change.starts_with("resources/assets/src") {
                    diff.build = true;
                } else if change.starts_with("database/migrations") {
                    diff.migration = true;
                }
            }
        }
    }

    diff
}

fn analyze_diff(
    Diff {
        build,
        yarn,
        composer,
        migration,
    }: Diff,
) -> String {
    if !build && !yarn && !composer && !migration {
        use rand::seq::IteratorRandom;

        let mut rng = rand::thread_rng();
        let choices = vec![
            "您可以直接拉取新 commit。",
            "还在犹豫什么，赶快拿起 <code>git pull</code> 订购吧！",
            "是朋友，就 <code>git pull</code>。介是你没有用过的船新版本。",
        ];
        return choices
            .into_iter()
            .choose(&mut rng)
            .map(|text| {
                let mut text = text.to_string();
                text.push('\n');
                text
            })
            .unwrap();
    }

    let front = if yarn {
        "先执行 <code>yarn</code>，然后执行 <code>pwsh ./scripts/build.ps1</code>。"
    } else if build {
        "执行 <code>pwsh ./scripts/build.ps1</code>。"
    } else {
        ""
    };

    let back = if composer {
        "执行 <code>composer install</code>。"
    } else {
        ""
    };

    let db = if migration {
        "执行 <code>php artisan migrate</code>。"
    } else {
        ""
    };

    let messages = vec![front, back, db]
        .into_iter()
        .filter(|msg| *msg != "")
        .join("\n");
    format!("拉取此 commit 后，您需要：\n{}", messages)
}

fn format_log(log: &[github::Commit]) -> String {
    log.iter()
        .map(|commit| {
            let message = commit.commit.message.replace('\n', " ");
            let mut sha = commit.sha.clone();
            sha.truncate(8);

            md2html(format!("**{}**: {}", sha, message))
        })
        .join("\n")
}

fn get_artifact_link(run_id: Option<String>) -> String {
    if let Some(run_id) = run_id {
        format!(
            "https://nightly.link/bs-community/blessing-skin-server/actions/runs/{}/artifact.zip",
            run_id
        )
    } else {
        "https://nightly.link/bs-community/blessing-skin-server/workflows/CI/dev/artifact.zip"
            .to_string()
    }
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
fn test_diff() {
    use github::compare::File;

    let files = &[File {
        filename: "tests".into(),
    }];
    let diff_result = diff(files);
    assert!(!diff_result.build);
    assert!(!diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "package.json".into(),
    }];
    let diff_result = diff(files);
    assert!(diff_result.build);
    assert!(diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "yarn.lock".into(),
    }];
    let diff_result = diff(files);
    assert!(diff_result.build);
    assert!(diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "composer.json".into(),
    }];
    let diff_result = diff(files);
    assert!(!diff_result.build);
    assert!(!diff_result.yarn);
    assert!(diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "composer.lock".into(),
    }];
    let diff_result = diff(files);
    assert!(!diff_result.build);
    assert!(!diff_result.yarn);
    assert!(diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "webpack.config.js".into(),
    }];
    let diff_result = diff(files);
    assert!(diff_result.build);
    assert!(!diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "resources/assets/src/index.ts".into(),
    }];
    let diff_result = diff(files);
    assert!(diff_result.build);
    assert!(!diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "resources/assets/test/setup.ts".into(),
    }];
    let diff_result = diff(files);
    assert!(!diff_result.build);
    assert!(!diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(!diff_result.migration);

    let files = &[File {
        filename: "database/migrations/a.php".into(),
    }];
    let diff_result = diff(files);
    assert!(!diff_result.build);
    assert!(!diff_result.yarn);
    assert!(!diff_result.composer);
    assert!(diff_result.migration);
}

#[test]
fn test_analyze_diff() {
    let mut diff = Diff::default();

    let analysis = analyze_diff(diff.clone());
    assert!(!analysis.contains("yarn"));
    assert!(!analysis.contains("pwsh"));
    assert!(!analysis.contains("composer"));

    diff.yarn = true;
    let analysis = analyze_diff(diff.clone());
    assert!(analysis.contains("yarn"));
    assert!(analysis.contains("pwsh"));

    diff.build = true;
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

    diff.composer = false;
    diff.migration = true;
    let analysis = analyze_diff(diff.clone());
    assert!(analysis.contains("php artisan migrate"));
}

#[test]
fn test_format_log() {
    use github::{Commit, CommitDetail};

    let log = &[
        Commit {
            sha: "123456789".into(),
            commit: CommitDetail {
                message: "kumiko".into(),
            },
        },
        Commit {
            sha: "987654321".into(),
            commit: CommitDetail {
                message: "reina".into(),
            },
        },
    ];

    let line1 = "<strong>12345678</strong>: kumiko";
    let line2 = "<strong>98765432</strong>: reina";
    let expected = format!("{}\n{}", line1, line2);

    let output = format_log(log);
    assert_eq!(&expected, &output);
}

#[test]
fn test_md2html() {
    assert_eq!("&lt;modal&gt;", &md2html(String::from("<modal>")));
    assert_eq!("&quot;text&quot;", &md2html(String::from("\"text\"")));
    assert_eq!("&amp;", &md2html(String::from("&")));
    assert_eq!("<strong>bold</strong>", &md2html(String::from("**bold**")));
}
