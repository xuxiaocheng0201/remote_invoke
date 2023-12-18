use std::io::Read;
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use variable_len_reader::primitive::write_bool;
use crate::AUTO_LAUNCHER;

pub async fn ping(_bytes: &mut impl Read) -> Result<BytesMut> {
    let launcher = AUTO_LAUNCHER.as_ref().map_err(|e| anyhow!("{}", e.to_string()))?;
    if !launcher.is_enabled()? {
        launcher.enable()?;
    }
    let mut writer = BytesMut::new().writer();
    write_bool(&mut writer, launcher.is_enabled()?)?;
    Ok(writer.into_inner())
}
