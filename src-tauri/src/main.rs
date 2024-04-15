// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use regex::Regex;
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::error::Error;
use std::process::Command;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 1. 通过 curl 获取接口地址
// 2. 获取接口地址后，请求数据
#[derive(Debug)]
struct LoadPageParam {
    name: String,
    page_id: String,
    ver: String,
}

fn get_pageid_and_ver_by_url(url: &str) -> LoadPageParam {
    let output = Command::new("curl").arg(url).output().expect("error");
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    let re = Regex::new(r"name=([^&]*)&pageId=([^&]*)&ver=([^&]*)").unwrap();
    let mut name = String::new();
    let mut page_id = String::new();
    let mut ver = String::new();
    // 在输入字符串中搜索匹配项
    if let Some(captures) = re.captures(&output_str) {
        // 提取匹配项中的值
        name = captures
            .get(1)
            .map_or(String::new(), |m| String::from(m.as_str()));
        page_id = captures
            .get(2)
            .map_or(String::new(), |m| String::from(m.as_str()));
        ver = captures
            .get(3)
            .map_or(String::new(), |m| String::from(m.as_str()));
    }

    let result = LoadPageParam { name, page_id, ver };

    result
}

#[tauri::command]
async fn get_dy_loadpage_url(url: &str) -> Result<String, ()> {
    let res = get_dy_loadpage_url_res(url).await;
    match res {
        Ok(s) => Ok(s),
        Err(e) => Ok(e.to_string()),
    }
}

async fn get_dy_loadpage_url_res(url: &str) -> Result<String, Box<dyn Error>> {
    let page_id_and_ver = get_pageid_and_ver_by_url(url);

    let mut headers = get_common_headers();
    headers.insert("authority", "butterfly.douyucdn.cn".parse().unwrap());
    headers.insert(
        "accept",
        "application/json, text/plain, */*".parse().unwrap(),
    );
    headers.insert("origin", "https://www.douyu.com".parse().unwrap());
    headers.insert("referer", "https://www.douyu.com/".parse().unwrap());

    let client = Client::builder().build().unwrap();
    let res = client
        .get(format!(
            "https://butterfly.douyucdn.cn/api/page/loadPage?name={}&pageId={}&ver={}&view=0",
            page_id_and_ver.name, page_id_and_ver.page_id, page_id_and_ver.ver
        ))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}

#[tauri::command]
async fn get_user_data(date: &str, rid: &str, token: &str) -> Result<String, ()> {
    let res = get_user_data_res(date, rid, token).await;
    match res {
        Ok(s) => Ok(s),
        Err(e) => Ok(e.to_string()),
    }
}

// 获取主播当天数据（播酱）
async fn get_user_data_res(date: &str, rid: &str, token: &str) -> Result<String, Box<dyn Error>> {
    let mut headers = get_common_headers();
    headers.insert(
        "Accept",
        "application/json, text/javascript, */*; q=0.01"
            .parse()
            .unwrap(),
    );
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert(
        "Content-Type",
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        "Referer",
        "https://www.bojianger.com/anchor-detail-total.html"
            .parse()
            .unwrap(),
    );
    headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
    headers.insert("token", token.parse().unwrap());

    let client = Client::builder().build().unwrap();
    let res = client
        .get(format!(
            "https://www.bojianger.com/data/api/auth/anchor_detail_total.do?date={}&rid={}",
            date, rid
        ))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}

fn get_common_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("Pragma", "no-cache".parse().unwrap());
    headers.insert(
        "sec-ch-ua",
        "\"Chromium\";v=\"122\", \"Not(A:Brand\";v=\"24\", \"Google Chrome\";v=\"122\""
            .parse()
            .unwrap(),
    );
    headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "cors".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
    headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());

    headers
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            get_user_data,
            get_dy_loadpage_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
