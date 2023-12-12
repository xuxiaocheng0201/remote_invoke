use anyhow::Result;
use tokio::time::Instant;

mod network;
mod image_grab;

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        let now = Instant::now();
        image_grab::capture(&"screens").await?;
        println!("{:?}", now.elapsed());
    }
    // Ok(())
}
