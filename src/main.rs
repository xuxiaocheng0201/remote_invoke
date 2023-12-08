use anyhow::Result;
use crate::network::try_select_link;

mod network;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{:?}", try_select_link(true).await?);
    Ok(())
}
