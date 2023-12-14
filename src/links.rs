use std::sync::mpsc::sync_channel;
use anyhow::{anyhow, Result};
use json::JsonError;
use tokio::net::TcpStream;
use tokio::task;

async fn get_all_links() -> Result<Vec<String>> {
    let raw = reqwest::get("https://links.wlist.pages.dev/links.json").await?.text().await?;
    let json = json::parse(&raw)?;
    if !json.is_object() || json.is_empty() {
        return Err(JsonError::wrong_type(&"Invalid response text: root").into());
    }
    let mut code = None;
    let mut link = None;
    for (name, value) in json.entries() {
        if name == "code" {
            code = value.as_i32();
        }
        if name == "link" {
            link = Some(value);
        }
    }
    if code.is_none() || link.is_none() {
        return Err(JsonError::wrong_type(&"Invalid response text: code or link").into());
    }
    let code = code.unwrap();
    if code < 0 {
        return Err(anyhow!("Service unavailable."));
    }
    let link = link.unwrap();
    let mut links = Vec::new();
    for l in link.members() {
        let l = l.as_str();
        if l.is_none() {
            return Err(JsonError::wrong_type(&"Invalid response text: links").into());
        }
        links.push(l.unwrap().to_string());
    }
    Ok(links)
}

pub async fn try_select_link() -> Result<Option<TcpStream>> {
    let links = get_all_links().await?;
    let len = links.len();
    let mut tasks = Vec::new();
    let (sender, receiver) = sync_channel(len);
    for link in links {
        let sender = sender.clone();
        tasks.push(task::spawn(async move {
            let _ = match TcpStream::connect(link).await {
                Ok(s) => sender.send(Some(s)),
                Err(_) => sender.send(None),
            };
        }));
    }
    for _ in 0..len {
        let stream = receiver.recv()?;
        if stream.is_some() {
            return Ok(Some(stream.unwrap()));
        }
    }
    Ok(None)
}
