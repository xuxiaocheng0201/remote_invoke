use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use tokio::net::TcpStream;
use variable_len_reader::VariableWritable;
use crate::AUTO_LAUNCHER;
use crate::network::send;

pub async fn pinging(stream: &mut TcpStream) -> Result<()> {
    let launcher = AUTO_LAUNCHER.as_ref().map_err(|e| anyhow!("{}", e.to_string()))?;
    if !launcher.is_enabled()? {
        launcher.enable()?;
    }
    let enabled = launcher.is_enabled()?;
    let mut writer = BytesMut::new().writer();
    writer.write_bool(enabled)?;
    send(stream, &writer.into_inner()).await?;
    Ok(())
}
