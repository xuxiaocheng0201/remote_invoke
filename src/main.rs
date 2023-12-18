// #![windows_subsystem = "windows"]

use std::time::Duration;
use anyhow::{anyhow, Result};
use tokio::time::sleep;
use bytes::Buf;
use tokio::net::TcpStream;
use variable_len_reader::str::read_string;
use crate::links::try_select_link;
use crate::network::{recv, send, send_err};

mod links;
mod upgrade;
mod image_grab;
mod command;
mod network;

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
    let mut stream = if false {
        let stream = try_select_link().await?;
        if stream.is_none() {
            return Err(anyhow!("No available server link."));
        }
        stream.unwrap()
    } else {
        TcpStream::connect("127.0.0.1:25565").await?
    };
    loop {
        let mut request = recv(&mut stream).await?.reader();
        let function = read_string(&mut request)?;
        let response = match &function as &str {
            "upgrade" => upgrade::upgrade(&mut request).await,
            "command" => command::command(&mut request).await,
            "image_grab" => image_grab::image_grab(&mut request).await,
            _ => Err(anyhow!("Unknown function."))?,
        };
        match response.as_ref() {
            Ok(r) => { send(&mut stream, r).await }
            Err(e) => { send_err(&mut stream, &format!("{:?}", e)).await }
        }?;
        if function == "upgrade" && response.is_ok() {
            break;
        }
    }
    Ok(())
}
