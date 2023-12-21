use std::fs::read;
use std::path::Path;
use anyhow::{anyhow, Result};
use screenshots::Screen;
use tempfile::tempdir;
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use variable_len_reader::VariableWritable;
use crate::network::send;

async fn capture_screens(directory: &Path) -> Result<()> {
    let mut tasks = JoinSet::<Result<()>>::new();
    for screen in Screen::all()? {
        let mut path = directory.to_path_buf();
        tasks.spawn(async move {
            let image = screen.capture()?;
            path.push(format!("{}.png", screen.display_info.id));
            image.save(path)?;
            Ok(())
        });
    }
    while let Some(res) = tasks.join_next().await {
        res??;
    }
    Ok(())
}

pub async fn capture(stream: &mut TcpStream) -> Result<()> {
    let temp = tempdir()?;
    let temp = temp.path();
    capture_screens(temp).await?;
    let mut images = Vec::new();
    for dir in temp.read_dir()? {
        images.push(dir?.path());
    }
    if images.is_empty() {
        return Err(anyhow!("No images."));
    }
    send(stream, |writer| {
        writer.write_u32_varint(images.len() as u32)?;
        for image in &images {
            let buffer = read(image)?;
            writer.write_u8_vec(&buffer)?;
        }
        Ok(())
    }).await
}
