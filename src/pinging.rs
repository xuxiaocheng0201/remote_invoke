pub use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use log::{debug, error, info};
use tokio::net::TcpStream;
use variable_len_reader::{VariableReadable, VariableWritable};
use crate::network::send_recv;

pub async fn pinging(client: &mut TcpStream) -> Result<()> {
    let mut buf = BytesMut::new().writer();
    buf.write_string(&"ping")?;
    debug!("Sending pinging request...");
    if let Some(response) = send_recv(client, &buf.into_inner()).await.unwrap() {
        let mut response = response.reader();
        if response.read_bool()? {
            info!("Client: AutoLaunch is enabled.");
        } else {
            error!("Client: AutoLaunch is failed to enable!");
        }
    }
    Ok(())
}
