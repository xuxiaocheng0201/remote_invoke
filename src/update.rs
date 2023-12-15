use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process::Command;
use anyhow::Result;
use bytes::{BufMut, BytesMut};
use futures_util::StreamExt;
use md5::{Digest, Md5};
use variable_len_reader::str::{read_string, write_string};

pub async fn update(bytes: &mut impl Read) -> Result<(BytesMut, bool)> {
    let url = read_string(bytes)?;
    let md5 = read_string(bytes)?;
    let mut stream = reqwest::get(url).await?.bytes_stream();
    let mut file = OpenOptions::new().write(true).read(true).create(true).truncate(true).open("update.exe")?;
    let mut hasher = Md5::default();
    while let Some(bin) = stream.next().await {
        let bin = bin?;
        file.write_all(&bin)?;
        hasher.update(&bin);
    }
    let hasher = format!("{:x}", hasher.finalize());
    let mut writer = BytesMut::new().writer();
    Ok(if md5 != hasher {
        write_string(&mut writer, &format!("Invalid md5 hash. Excepted {}, got {}.", md5, hasher))?;
        (writer.into_inner(), false)
    } else {
        write_string(&mut writer, &"Download successfully.")?;
        Command::new("update.exe").spawn()?;
        (writer.into_inner(), true)
    })
}
