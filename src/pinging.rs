use anyhow::{anyhow, Result};
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
    send(stream, |writer| {
        writer.write_bool(enabled)?;
        Ok(())
    }).await?;
    Ok(())
}
