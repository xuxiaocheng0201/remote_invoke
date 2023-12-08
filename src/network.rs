use std::sync::mpsc::sync_channel;
use anyhow::Result;
use json::JsonError;
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
        return Ok(Vec::new());
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

pub async fn try_select_link(dangerous: bool) -> Result<Option<String>> {
    let links = get_all_links().await?;
    let len = links.len();
    let mut tasks = Vec::new();
    let (sender, receiver) = sync_channel(len);
    let client = reqwest::Client::builder().danger_accept_invalid_certs(dangerous).build()?;
    for link in links {
        let client = client.clone();
        let sender = sender.clone();
        tasks.push(task::spawn(async move {
            let _ = if match client.get(link.to_owned() + "/ping").send().await {
                Ok(r ) => r.text().await.is_ok(),
                Err(e) => { println!("{:?}", e); false },
            } { sender.send(Some(link)) } else { sender.send(None) };
        }));
    }
    for _ in 0..len {
        let a = receiver.recv()?;
        if a.is_some() {
            return Ok(Some(a.unwrap() + "/"));
        }
    }
    Ok(None)
}
