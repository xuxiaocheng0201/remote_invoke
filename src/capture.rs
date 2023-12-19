use std::fs::read;
use std::io::Read;
use std::path::Path;
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use screenshots::Screen;
use tempfile::tempdir;
use tokio::task::JoinSet;
use variable_len_reader::VariableWritable;
use variable_len_reader::varint::VarintWriter;

async fn ca(directory: &Path) -> Result<()> {
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

pub async fn capture(_bytes: &mut impl Read) -> Result<BytesMut> {
    let temp = tempdir()?;
    let temp = temp.path();
    ca(temp).await?;
    let mut images = Vec::new();
    for dir in temp.read_dir()? {
        images.push(dir?.path());
    }
    if images.is_empty() {
        return Err(anyhow!("No images."));
    }
    let mut writer = BytesMut::new().writer();
    writer.write_usize_varint(images.len())?;
    for image in images {
        let buffer = read(image)?;
        writer.write_u8_vec(&buffer)?;
    }
    Ok(writer.into_inner())
}
