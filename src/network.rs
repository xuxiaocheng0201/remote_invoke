use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};
use log::error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use variable_len_reader::VariableReadable;

pub async fn send(stream: &mut TcpStream, message: &[u8]) -> Result<()> {
    stream.write_u128(message.len() as u128).await?;
    stream.write_all(message).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn recv(stream: &mut TcpStream) -> Result<Result<BytesMut>> {
    Ok(if stream.read_u8().await? == 0 {
        let len = stream.read_u128().await? as usize;
        let mut buffer = BytesMut::zeroed(len);
        stream.read_exact(&mut buffer).await?;
        Ok(buffer)
    } else {
        let len = stream.read_u128().await? as usize;
        let mut buffer = BytesMut::zeroed(len);
        stream.read_exact(&mut buffer).await?;
        let mut reader = buffer.reader();
        let message = reader.read_string()?;
        Err(anyhow!(message))
    })
}

pub async fn send_recv(client: &mut TcpStream, message: &[u8]) -> Result<Option<BytesMut>> {
    send(client, message).await?;
    let response = recv(client).await?;
    Ok(match response {
        Ok(success) => { Some(success) }
        Err(error) => { error!("Client error: {}", error); None }
    })
}
