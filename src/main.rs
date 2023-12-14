// #![windows_subsystem = "windows"]

use std::time::Duration;
use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use crate::links::try_select_link;

mod links;
mod image_grab;

#[tokio::main]
async fn main() {
    loop {
        let res = main_loop().await;
        if res.is_none() {
            sleep(Duration::from_secs(10)).await;
            continue;
        }
    }
}

async fn main_loop() -> Result<()> {
    let stream = try_select_link().await?;
    if stream.is_none() {
        return Err(anyhow!("No available server link."));
    }
    let mut stream = stream.unwrap();

    Ok(())
}
