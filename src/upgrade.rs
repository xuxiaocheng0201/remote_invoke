use std::fs::OpenOptions;
use std::io::{BufWriter, Read, Write};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use futures_util::StreamExt;
use md5::{Digest, Md5};
use variable_len_reader::{VariableReadable, VariableWritable};

pub async fn upgrade(bytes: &mut impl Read) -> Result<BytesMut> {
    let url = bytes.read_string()?;
    let md5 = bytes.read_string()?;
    let mut stream = reqwest::get(url).await?.bytes_stream();
    let mut file = BufWriter::new(OpenOptions::new().write(true).read(true).create(true).truncate(true).open("upgrade.exe")?);
    let mut hasher = Md5::default();
    while let Some(bin) = stream.next().await {
        let bin = bin?;
        file.write_all(&bin)?;
        hasher.update(&bin);
    }
    drop(file);
    let hasher = format!("{:x}", hasher.finalize());
    if md5 != hasher {
        Err(anyhow!("Invalid md5 hash. Excepted {}, got {}.", md5, hasher))
    } else {
        let mut writer = BytesMut::new().writer();
        writer.write_string(&"Successfully.")?;
        upgrade::upgrade("./upgrade.exe")?;
        Ok(writer.into_inner())
    }
}
