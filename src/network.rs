use anyhow::Result;
use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use variable_len_reader::asynchronous::{AsyncVariableReadable, AsyncVariableWritable};

pub async fn send(stream: &mut TcpStream, message: &[u8]) -> Result<()> {
    stream.write_bool(true).await?;
    stream.write_u128_varint(message.len() as u128).await?;
    stream.write_more(&message).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn recv(stream: &mut TcpStream) -> Result<BytesMut> {
    let len = stream.read_u128_varint().await? as usize;
    let mut buffer = BytesMut::zeroed(len);
    stream.read_more(&mut buffer).await?;
    Ok(buffer)
}

pub async fn send_err(stream: &mut TcpStream, message: &str) -> Result<()> {
    stream.write_bool(false).await?;
    stream.write_string(message).await?;
    stream.flush().await?;
    Ok(())
}
