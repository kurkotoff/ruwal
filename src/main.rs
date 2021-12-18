#[macro_use] extern crate text_io;
use reqwest;
use tokio;
use std::fs::File;
use std::rc::Rc;
use std::io::Write;
use subprocess;
use subprocess::Exec;
use colored::*;

struct MyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let v = query_wal().await;
    display_options(&v);
    let (choice_url, choice_name) = user_choice(&v).unwrap();
    apply_wal(choice_url, choice_name).await?;
    Ok(())
}

async fn query_wal() -> serde_json::Value {
    let resp = reqwest::get("https://wallhaven.cc/api/v1/search")
        .await.unwrap()
        .text()
        .await.unwrap()
        .replace("\\", "");

    serde_json::from_str(&resp).unwrap()
}

fn display_options(v: &serde_json::Value) {
    let size = opt_size(&v);
    for i in 0..size {
        let colors = img_colors(v, &i);
        let mut tcolors = Vec::new();
        for i in 0..5 {
            tcolors.push(hex_to_rgb(&colors[i]));
        }
        println!(
            "{}: {}     {}{}{}{}{}   ",
            i, img_name(&v, &i),
            "███".truecolor(tcolors[0].r, tcolors[0].g, tcolors[0].b),
            "███".truecolor(tcolors[1].r, tcolors[1].g, tcolors[1].b),
            "███".truecolor(tcolors[2].r, tcolors[2].g, tcolors[2].b),
            "███".truecolor(tcolors[3].r, tcolors[3].g, tcolors[3].b),
            "███".truecolor(tcolors[4].r, tcolors[4].g, tcolors[4].b),
        );
    }
}

fn opt_size(value: &serde_json::Value) -> usize {
    value["data"].as_array().unwrap().len()
}

fn img_url(value: &serde_json::Value, idx: &usize) -> String {
    let res = Rc::new(value["data"][idx]["path"].as_str().unwrap());
    return res.to_string()
}

fn img_name(value: &serde_json::Value, idx: &usize) -> String {
    Rc::new(img_url(value, idx).split("/").last().unwrap()).to_string()
}

fn img_colors(value: &serde_json::Value, idx: &usize) -> Vec<String> {
    let mut res:Vec<String> = Vec::new();
    let colors = value["data"][idx]["colors"].as_array().unwrap();
    for i in 0..colors.len() {
        res.push(colors[i].as_str().unwrap().to_string());
    }
    res
}

fn user_choice<'a>(value: &'a serde_json::Value) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut choice: usize = 100;
    let size = opt_size(&value);
    while choice > size {
        println!("Select option 0 to {}", size - 1);
        choice = read!();
    }
    let url = img_url(&value, &choice);
    let name = img_name(&value, &choice);

    Ok((url, name))
}

async fn apply_wal(url: String, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = reqwest::get(url)
        .await?
        .bytes()
        .await?;
    let mut file = File::create(&name)?;
    file.write_all(&bytes)?;

    Exec::cmd(format!("wal"))
        .args(&["-i", format!("{}", name).as_str()])
        .capture()?;

    Ok(())
}

fn hex_to_rgb(color: &String) -> MyColor {
    let mut color_num: u64 = u64::from_str_radix(color.trim_start_matches("#"), 16).unwrap();
    // r g b
    let mut res = MyColor {r: 0, g: 0, b: 0};
    res.b = (color_num % 256) as u8; color_num /= 256;
    res.g = (color_num % 256) as u8; color_num /= 256;
    res.r = (color_num % 256) as u8;
    res
}