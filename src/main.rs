use std::io::Read;

use clap::{command, Parser};
use serde::{Deserialize, Serialize};
use url::Url;
use wl_clipboard_rs::paste::{get_contents, self};

pub static SHLINK_URL: &str = "https://l.davidon.top";
pub static SHLINK_API_KEY: &str = "26bef2a8-eda5-4660-91cc-2994a4bc2173";
pub static SHLINK_DOMAIN: &str = "l.davidon.top";

#[derive(Parser, Debug)]
#[command(author = "DavidOnTop", version = "0.1.0", about = "A cli tool to make short urls using shlink", long_about = None)]
struct Args {
    /// number type
    #[arg(short, long)]
    max_visits: Option<u32>,
    /// list type
    #[arg(long)]
    tags: Option<Vec<String>>,
    /// string type
    #[arg(long)]
    title: Option<String>,
    #[arg(short, long)]
    crawlable: Option<bool>,
    #[arg(short = 'q', long)]
    forward_query: Option<bool>,
    /// string type
    #[arg(short = 's', long)]
    custom_slug: Option<String>,
    #[arg(short, long)]
    find_if_exists: Option<bool>,
    /// string type
    #[arg(short, long)]
    domain: Option<String>,
}

#[derive(Serialize, Debug)]
struct Req {
    #[serde(rename = "longUrl")]
    long_url: String,
    #[serde(rename = "customSlug", skip_serializing_if = "Option::is_none")]
    custom_slug: Option<String>,
    #[serde(rename = "maxVisits", skip_serializing_if = "Option::is_none")]
    max_visits: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    crawlable: Option<bool>,
    #[serde(rename = "forwardQuery")]
    forward_query: bool,
    #[serde(rename = "findIfExists")]
    find_if_exists: bool,
    domain: String,
}

#[derive(Deserialize, Debug)]
struct Res {
    #[serde(rename = "shortUrl")]
    short_url: String,
}

fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let args = Args::parse();
    tracing::info!("{:?}", args);

    let clipboard_res = get_contents(paste::ClipboardType::Regular, paste::Seat::Unspecified, paste::MimeType::Text);
    let clipboard = match clipboard_res {
        Ok((mut pipe, _)) => {
            let mut contents = String::new();
            pipe.read_to_string(&mut contents).unwrap();
            contents
        },
        Err(e) => panic!("Error: {}", e)
    };
    let clipboard = clipboard.split("\n").collect::<Vec<&str>>()[0].to_string();
    tracing::info!("Clipboard: {}", clipboard);
    let url = Url::parse(clipboard.as_str()).unwrap();
    assert!(url.scheme() == "http" || url.scheme() == "https", "Clipboard does not contain a valid url");
    assert!(url.host().is_some(), "Clipboard does not contain a valid url");

    let req = Req {
        long_url: clipboard,
        custom_slug: args.custom_slug,
        max_visits: args.max_visits,
        tags: args.tags,
        title: args.title,
        crawlable: args.crawlable,
        forward_query: args.forward_query.unwrap_or(true),
        find_if_exists: args.find_if_exists.unwrap_or(true),
        domain: args.domain.unwrap_or(SHLINK_DOMAIN.to_string()),
    };
    tracing::info!("Req: {:?}", serde_json::to_string(&req).unwrap());

    let client = reqwest::blocking::Client::new();
    let res: Res = client.post(format!("{}/rest/v3/short-urls", SHLINK_URL)).header("X-Api-Key", SHLINK_API_KEY).json(&req).send().unwrap().json().unwrap();
    tracing::info!("Res: {:?}", res);

    let cmd = format!("wl-copy {}", res.short_url);
    tracing::info!("Cmd: {}", cmd);
    std::process::Command::new("sh").arg("-c").arg(cmd).spawn().unwrap();
}
