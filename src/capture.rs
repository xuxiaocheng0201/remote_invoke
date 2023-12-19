use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::process::Command;
use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use log::{debug, info};
use tokio::net::TcpStream;
use variable_len_reader::str::{read_u8_vec, write_string};
use variable_len_reader::variable_len::read_variable_u32;
use crate::network::send_recv;
use crate::read_line;

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
