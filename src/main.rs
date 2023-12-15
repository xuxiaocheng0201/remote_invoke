mod behaviour;

use std::env;
use std::io::stdin;
use std::process::Command;
use anyhow::Result;
use log::info;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()>{
    env_logger::init();
    let host = env::var("host").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("port").map_or(Ok(25565), |port| port.parse())?;
    info!("Waiting for client connect on {}:{}...", host, port);
    let server = TcpListener::bind((host, port)).await?;
    let (mut client, address) = server.accept().await?;
    info!("Client connected from {}.", address);
    loop {
        println!("\
    Please enter the operation code:
        0: cls
        1: exit
        2: update
        3: capture
    ");
        let mut operation_code = String::new();
        stdin().read_line(&mut operation_code)?;
        match operation_code.trim().parse() {
            Ok(0) => { Command::new("cmd").arg("/c").arg("cls").spawn()?.wait()?; },
            Ok(1) => { break },
            Ok(2) => { behaviour::update(&mut client).await?; },
            Ok(3) => { behaviour::capture(&mut client).await?; },
            _ => { println!("Invalid operation code.") },
        }
    }
    Ok(())
}
