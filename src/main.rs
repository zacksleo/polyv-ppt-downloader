use serde::{Deserialize, Serialize};
use std::error;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

use clap::{AppSettings, Clap};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "zacksleo. <zacksleo@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long, default_value = "default.json")]
    json: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    file_name: String,
    convert_file_json: FileJson,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileJson {
    image_count: u32,
    images: Vec<String>,
}

async fn download(target: &str) -> Result<()> {
    let response = reqwest::get(target).await?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        File::create(fname)?
    };
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;
    Ok(())
}

fn read_to_string(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let contents = read_to_string(&opts.json)?;
    let response: Response = serde_json::from_str(&contents)?;
    for image in response.convert_file_json.images {
        println!("下载文件：{}", image);
        download(image.as_str()).await?;
    }
    Ok(())
}
