use crate::bot::{Bot};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::AsRef;
use std::path::Path;
use tokio::fs;

#[derive(Deserialize, Serialize)]
struct Plugin {
    name: String,
    version: String,
}

type PluginsList = Vec<Plugin>;

async fn fetch<P: AsRef<Path>>(path: P) -> anyhow::Result<PluginsList> {
    let content = fs::read(path).await?;
    let list = serde_json::from_slice::<PluginsList>(&content)?;

    Ok(list)
}

pub async fn execute<P: AsRef<Path>>(bot: &Bot, path: P) -> anyhow::Result<()> {
    let list = fetch(path).await?;
    if list.is_empty() {
        info!("No plugins have been updated.");
        return Ok(());
    }

    let list = list
        .into_iter()
        .map(|Plugin { name, version }| format!("• *{}* 已更新至 {}", name, version))
        .join("\n");
    let text = format!("插件更新：\n{}", list);

    bot.send_message(text, None)
        .await
        .map_err(anyhow::Error::from)
}

#[tokio::test]
async fn test_fetch() {
    use tokio::io::AsyncWriteExt;

    let data = vec![
        Plugin {
            name: "kumiko".to_string(),
            version: "1.2.3".to_string(),
        },
        Plugin {
            name: "reina".to_string(),
            version: "4.5.6".to_string(),
        },
    ];
    let json = serde_json::to_vec(&data).unwrap();

    let mut path = std::env::temp_dir();
    path.push("plugins.json");

    {
        let mut file = fs::File::create(&path).await.unwrap();
        file.write_all(&*json).await.unwrap();
    }

    let list = fetch(&path).await.unwrap();
    assert_eq!(list[0].name, "kumiko");
    assert_eq!(list[0].version, "1.2.3");
    assert_eq!(list[1].name, "reina");
    assert_eq!(list[1].version, "4.5.6");

    fs::remove_file(&path).await.unwrap();
}
