use anyhow::Result;
use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use variable_len_reader::str::write_string;

pub async fn send(stream: &mut TcpStream, message: &[u8]) -> Result<()> {
    stream.write_u8(0).await?;
    stream.write_u128(message.len() as u128).await?;
    stream.write_all(message).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn send_err(stream: &mut TcpStream, message: &str) -> Result<()> {
    stream.write_u8(1).await?;
    let mut writer = BytesMut::new().writer();
    write_string(&mut writer, message)?;
    let buffer = writer.into_inner();
    stream.write_u128(buffer.len() as u128).await?;
    stream.write_all(&buffer).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn recv(stream: &mut TcpStream) -> Result<BytesMut> {
    let len = stream.read_u128().await? as usize;
    let mut buffer = BytesMut::zeroed(len);
    stream.read_exact(&mut buffer).await?;
    Ok(buffer)
}
