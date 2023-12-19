use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use log::{debug, info};
use tokio::net::TcpStream;
use variable_len_reader::{VariableReadable, VariableWritable};
use variable_len_reader::varint::VarintReader;
use crate::network::send_recv;
use crate::read_line;

pub async fn capture(client: &mut TcpStream) -> Result<()> {
    let mut buf = BytesMut::new().writer();
    buf.write_string(&"image_grab")?;
    debug!("Sending capture request...");
    if let Some(response) = send_recv(client, &buf.into_inner()).await? {
        let mut response = response.reader();
        let count = response.read_usize_varint()?;
        info!("Received {} images.", count);
        create_dir_all("images")?;
        for i in 0..count {
            let image = response.read_u8_vec()?;
            let mut file = BufWriter::new(File::create(format!("images/{}.png", i))?);
            file.write_all(&image)?;
        }
        println!("Press 'ok' to open the images folder:");
        if read_line()?.trim().to_lowercase() == "ok" {
            open::that("images")?;
        }
    }
    Ok(())
}
