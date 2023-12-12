use anyhow::Result;
use screenshots::Screen;
use tokio::task::JoinSet;

pub async fn capture(directory: &'static str) -> Result<()> {
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
