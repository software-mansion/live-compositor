use anyhow::{anyhow, Result};

use log::error;
use reqwest::{blocking::Response, StatusCode};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use video_compositor::config::config;

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
        .post(format!("http://127.0.0.1:{}/--/api", config().api_port))
        .timeout(Duration::from_secs(100))
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

    let formated_body = get_formated_body(&body_str);
    error!(
        "Request failed:\n\nRequest: {}\n\nResponse code: {}\n\nResponse body:\n{}",
        request_str, status, formated_body
    )
}

fn get_formated_body(body_str: &str) -> String {
    let Ok(mut body_json) = serde_json::from_str::<serde_json::Value>(body_str) else {
        return body_str.to_string();
    };

    let Some(stack_value) = body_json.get("stack") else {
        return serde_json::to_string_pretty(&body_json).unwrap();
    };

    let errors: Vec<&str> = stack_value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let msg_string = " - ".to_string() + &errors.join("\n - ");
    let body_map = body_json.as_object_mut().unwrap();
    body_map.remove("stack");
    format!(
        "{}\n\nError stack:\n{}",
        serde_json::to_string_pretty(&body_map).unwrap(),
        msg_string,
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
