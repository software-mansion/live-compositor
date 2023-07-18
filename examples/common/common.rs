use anyhow::{anyhow, Result};
use reqwest::{blocking::Response, StatusCode};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;

/// The SDP file will describe an RTP session on localhost with H264 encoding.
pub fn write_example_sdp_file(port: u16) -> Result<String> {
    let sdp_filepath = PathBuf::from(format!("/tmp/example_sdp_input_{}.sdp", port));
    let mut file = File::create(&sdp_filepath)?;
    file.write_all(
        format!(
            "\
                    v=0\n\
                    o=- 0 0 IN IP4 127.0.0.1\n\
                    s=No Name\n\
                    c=IN IP4 127.0.0.1\n\
                    m=video {} RTP/AVP 96\n\
                    a=rtpmap:96 H264/90000\n\
                    a=fmtp:96 packetization-mode=1\n\
                    a=rtcp-mux\n\
                ",
            port
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
    let response = client.post("http://127.0.0.1:8001").json(json).send()?;
    if response.status() >= StatusCode::BAD_REQUEST {
        return Err(anyhow!(
            "Request failed: \n\trequest: {:?}\n\tresponse: {:?}",
            serde_json::to_string(json)?,
            response.text()
        ));
    }
    Ok(response)
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
