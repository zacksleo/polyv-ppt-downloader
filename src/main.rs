use printpdf::image::GenericImageView;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::error;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;
use std::path::Path;

use printpdf::*;
use std::io::BufWriter;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

use clap::{AppSettings, Clap};

const DPI_RATE: f64 = 0.084666836;

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

fn parse_fname_from_url(url: &str) -> Result<String> {
    let url = Url::parse(&url).unwrap();
    let fname = url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");
    Ok(fname.to_owned())
}

async fn download(target: &str) -> Result<()> {
    let fname = parse_fname_from_url(target).unwrap();
    println!("fname = {}", fname);
    if Path::new(&fname).exists() {
        println!("文件已存在，跳过...");
        return Ok(());
    }
    let response = reqwest::get(target).await?;

    let mut dest = { File::create(fname)? };
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;
    Ok(())
}

// 读取文件中的字符
fn read_to_string(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// 读取图片大小(转成毫米)
fn get_img_mm_size(img: &str) -> (f64, f64) {
    let img = image::open(&Path::new(&img)).unwrap();
    let (width, height) = img.dimensions();
    (width as f64 * DPI_RATE, height as f64 * DPI_RATE)
}

// 将图片合并成 PDF
fn merge_to_pdf(title: &str, images: Vec<String>) {
    let (width, height) = get_img_mm_size(&images[0]);
    let (doc, mut page, mut layer) =
        PdfDocument::new("PDF_Document_title", Mm(width), Mm(height), "Layer 1");

    for (index, image) in images.iter().enumerate() {
        if index > 0 {
            let (page1, layer1) = doc.add_page(Mm(width), Mm(height), "Page 2, Layer 1");
            page = page1;
            layer = layer1;
        }
        let mut image_file = File::open(&Path::new(&image)).unwrap();
        let image =
            Image::try_from(image::jpeg::JpegDecoder::new(&mut image_file).unwrap()).unwrap();

        let current_layer = doc.get_page(page).get_layer(layer);
        image.add_to_layer(current_layer.clone(), None, None, None, None, None, None);
    }

    doc.save(&mut BufWriter::new(
        File::create(title).unwrap(),
    ))
    .unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let contents = read_to_string(&opts.json)?;
    let response: Response = serde_json::from_str(&contents)?;
    for image in &response.convert_file_json.images {
        println!("下载文件：{}", image);
        download(image.as_str()).await?;
    }

    let imgs = response
        .convert_file_json
        .images
        .into_iter()
        .map(|e| parse_fname_from_url(&e).unwrap())
        .collect();
    merge_to_pdf(&response.file_name, imgs);

    Ok(())
}
