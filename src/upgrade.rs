use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use futures_util::StreamExt;
use md5::{Digest, Md5};
use tokio::net::TcpStream;
use variable_len_reader::{VariableReadable, VariableWritable};
use crate::network::{recv, send};

pub async fn upgrade(stream: &mut TcpStream) -> Result<()> {
    let mut reader = recv(stream).await?.reader();
    let url = reader.read_string()?;
    let md5 = reader.read_string()?;
    let mut upgrader = reqwest::get(url).await?.bytes_stream();
    let mut file = BufWriter::new(OpenOptions::new()
        .write(true).create(true).truncate(true).open("upgrade.exe")?);
    let mut hasher = Md5::default();
    while let Some(bin) = upgrader.next().await {
        let bin = bin?;
        file.write_all(&bin)?;
        hasher.update(&bin);
    }
    drop(file);
    let hasher = format!("{:x}", hasher.finalize());
    if md5 != hasher {
        return Err(anyhow!("Invalid md5 hash. Excepted {}, got {}.", md5, hasher));
    }
    upgrade::upgrade("upgrade.exe")?;
    let mut writer = BytesMut::new().writer();
    writer.write_string(&"Successfully.")?;
    send(stream, &writer.into_inner()).await?;
    Ok(())
}
