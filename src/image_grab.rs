use std::path::Path;
use anyhow::Result;
use screenshots::Screen;
use tokio::fs::create_dir_all;
use tokio::task::JoinSet;

pub async fn capture(directory: &'static str) -> Result<()> {
    let path = Path::new(directory);
    if !path.is_dir() {
        create_dir_all(path).await?;
    }
    let mut tasks = JoinSet::new();
    for screen in Screen::all()? {
        tasks.spawn(async move {
            screen.capture()?.save(format!("{}/{}.png", directory, screen.display_info.id))?;
            Ok(())
        });
    }
    while let Some(res) = tasks.join_next().await {
        let res: Result<()> = res?;
        res?;
    }
    Ok(())
}
