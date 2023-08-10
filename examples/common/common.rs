use anyhow::{anyhow, Result};

use log::error;
use reqwest::{blocking::Response, StatusCode};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;

/// The SDP file will describe an RTP session on localhost with H264 encoding.
pub fn write_example_sdp_file(ip: &str, port: u16) -> Result<String> {
    let sdp_filepath = PathBuf::from(format!("/tmp/example_sdp_input_{}.sdp", port));
    let mut file = File::create(&sdp_filepath)?;
    file.write_all(
        format!(
            "\
                    v=0\n\
                    o=- 0 0 IN IP4 {}\n\
                    s=No Name\n\
                    c=IN IP4 {}\n\
                    m=video {} RTP/AVP 96\n\
                    a=rtpmap:96 H264/90000\n\
                    a=fmtp:96 packetization-mode=1\n\
                    a=rtcp-mux\n\
                ",
            ip, ip, port
        )
        .as_bytes(),
    )?;
    Ok(String::from(
        sdp_filepath
            .to_str()
            .ok_or_else(|| anyhow!("invalid utf string"))?,
    ))
}

pub fn post<T: Serialize + ?Sized>(json: &T) -> Result<Response> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://127.0.0.1:8001")
        .json(json)
        .send()
        .unwrap();
    if response.status() >= StatusCode::BAD_REQUEST {
        log_request_error(&json, response);
        return Err(anyhow!("request failed"));
    }
    Ok(response)
}

fn log_request_error<T: Serialize + ?Sized>(request_body: &T, response: Response) {
    let status = response.status();
    let request_str = serde_json::to_string_pretty(request_body).unwrap();
    let body_str = response.text().unwrap();
    let formated_body = serde_json::from_str::<serde_json::Value>(&body_str)
        .map(|parsed| serde_json::to_string_pretty(&parsed).unwrap())
        .unwrap_or(body_str);
    error!(
        "Request failed:\nRequest: {}\nResponse code: {}\nResponse body:\n{}",
        request_str, status, formated_body
    )
}

#[allow(dead_code)]
pub fn download(url: &str, destination: &Path) -> Result<()> {
    let mut resp = reqwest::blocking::get(url)?;
    let mut out = File::create(destination)?;
    io::copy(&mut resp, &mut out)?;
    Ok(())
}

#[allow(dead_code)]
pub fn ensure_downloaded(url: &str, destination: &Path) -> Result<()> {
    if destination.exists() {
        return Ok(());
    }
    download(url, destination)
}
