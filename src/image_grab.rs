use std::fs::read;
use std::io::Read;
use std::path::Path;
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use screenshots::Screen;
use tempfile::tempdir;
use tokio::task::JoinSet;
use variable_len_reader::str::write_u8_vec;
use variable_len_reader::variable_len::write_variable_u32;

async fn capture(directory: &Path) -> Result<()> {
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

pub async fn image_grab(_bytes: &mut impl Read) -> Result<BytesMut> {
    let temp = tempdir()?;
    let temp = temp.path();
    capture(temp).await?;
    let mut images = Vec::new();
    for dir in temp.read_dir()? {
        images.push(dir?.path());
    }
    if images.is_empty() {
        return Err(anyhow!("No images."));
    }
    let mut writer = BytesMut::new().writer();
    write_variable_u32(&mut writer, images.len() as u32)?;
    for image in images {
        let buffer = read(image)?;
        write_u8_vec(&mut writer, &buffer)?;
    }
    Ok(writer.into_inner())
}
