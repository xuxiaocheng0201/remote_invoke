use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use futures_util::StreamExt;
use log::{debug, error, info};
use md5::{Digest, Md5};
use tokio::net::TcpStream;
use variable_len_reader::{VariableReadable, VariableWritable};
use crate::network::send_recv;
use crate::read_line;

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
    buf.write_string(&"upgrade")?;
    buf.write_string(&url)?;
    buf.write_string(&md5)?;
    debug!("Sending upgrade request...");
    if let Some(response) = send_recv(client, &buf.into_inner()).await? {
        let mut response = response.reader();
        let message = response.read_string()?;
        println!("Client message: {}", message);
    }
    Ok(())
}
