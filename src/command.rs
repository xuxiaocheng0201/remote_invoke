use std::io::Read;
use anyhow::Result;
use bytes::BytesMut;

pub async fn command(_bytes: &mut impl Read) -> Result<BytesMut> {
    todo!()
}
