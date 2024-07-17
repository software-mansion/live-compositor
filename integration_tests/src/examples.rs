use anyhow::{anyhow, Result};

use futures_util::{SinkExt, StreamExt};
use live_compositor::{config::read_config, server};
use log::error;
use reqwest::{blocking::Response, StatusCode};
use std::{
    env,
    fs::{self, File},
    io,
    path::PathBuf,
    process, thread,
    time::{Duration, Instant},
};
use tokio_tungstenite::tungstenite;
use tracing::info;

use serde::Serialize;

pub fn post<T: Serialize + ?Sized>(route: &str, json: &T) -> Result<Response> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!(
            "http://127.0.0.1:{}/api/{}",
            read_config().api_port,
            route
        ))
        .timeout(Duration::from_secs(100))
        .json(json)
        .send()
        .unwrap();
    if response.status() >= StatusCode::BAD_REQUEST {
        log_request_error(&json, response);
        return Err(anyhow!("Request failed."));
    }
    Ok(response)
}

pub fn run_example(client_code: fn() -> Result<()>) {
    thread::spawn(move || {
        ffmpeg_next::format::network::init();

        download_all_assets().unwrap();

        if let Err(err) = wait_for_server_ready(Duration::from_secs(10)) {
            error!("{err}");
            process::exit(1);
        }

        thread::spawn(move || {
            if let Err(err) = client_code() {
                error!("{err}");
                process::exit(1);
            }
        });

        start_server_msg_listener();
    });
    server::run();
}

fn wait_for_server_ready(timeout: Duration) -> Result<()> {
    let server_status_url = "http://127.0.0.1:8081/status";
    let wait_start_time = Instant::now();
    loop {
        match reqwest::blocking::get(server_status_url) {
            Ok(_) => break,
            Err(_) => info!("Waiting for the server to start."),
        };
        if wait_start_time.elapsed() > timeout {
            return Err(anyhow!("Error while starting server, timeout exceeded."));
        }
        thread::sleep(Duration::from_millis(1000));
    }
    Ok(())
}

pub fn start_server_msg_listener() {
    thread::Builder::new()
        .name("Websocket Thread".to_string())
        .spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { server_msg_listener().await });
        })
        .unwrap();
}

async fn server_msg_listener() {
    let url = format!("ws://127.0.0.1:{}/ws", read_config().api_port);

    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("Failed to connect");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (mut outgoing, mut incoming) = ws_stream.split();

    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let tungstenite::Message::Close(None) = &msg {
                let _ = outgoing.send(msg).await;
                return;
            }
            match outgoing.send(msg).await {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = outgoing.send(tungstenite::Message::Close(None)).await;
                    return;
                }
            }
        }
    });

    let receiver_task = tokio::spawn(async move {
        while let Some(result) = incoming.next().await {
            match result {
                Ok(tungstenite::Message::Close(_)) => {
                    let _ = tx.send(tungstenite::Message::Close(None));
                    return;
                }
                Ok(tungstenite::Message::Ping(data)) => {
                    if tx.send(tungstenite::Message::Pong(data)).is_err() {
                        return;
                    }
                }
                Err(_) => {
                    let _ = tx.send(tungstenite::Message::Close(None));
                    return;
                }
                _ => {
                    info!("Received compositor event: {:?}", result);
                }
            }
        }
    });

    sender_task.await.unwrap();
    receiver_task.await.unwrap();
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

pub enum TestSample {
    /// 10 minute animated video with sound
    BigBuckBunny,
    /// 11 minute animated video with sound
    ElephantsDream,
    /// 28 sec video with no sound
    Sample,
    /// looped 28 sec video with no sound
    SampleLoop,
    /// generated sample video with no sound (also with second timer when using ffmpeg)
    Generic,
}

#[derive(Debug)]
struct AssetData {
    url: String,
    path: PathBuf,
}

fn download_all_assets() -> Result<()> {
    let assets = [AssetData {
        url: String::from("https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"),
        path: examples_root_dir().join("examples/assets/BigBuckBunny.mp4"),
    },
    AssetData {
        url: String::from("http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4"),
        path: examples_root_dir().join("examples/assets/ElephantsDream.mp4"),
    },
    AssetData {
        url: String::from("https://filesamples.com/samples/video/mp4/sample_1280x720.mp4"),
        path: examples_root_dir().join("examples/assets/sample_1280_720.mp4"),
    }];

    let any_asset_downloaded = assets
        .iter()
        .map(download_asset)
        .any(|download_result| download_result.is_ok());

    if !any_asset_downloaded {
        return Err(anyhow!(
            "Error while downloading assets, couldn't download any"
        ));
    }
    Ok(())
}

fn map_asset_to_path(asset: &TestSample) -> Option<PathBuf> {
    match asset {
        TestSample::BigBuckBunny => {
            Some(examples_root_dir().join("examples/assets/BigBuckBunny.mp4"))
        }
        TestSample::ElephantsDream => {
            Some(examples_root_dir().join("examples/assets/ElephantsDream.mp4"))
        }
        TestSample::Sample | TestSample::SampleLoop => {
            Some(examples_root_dir().join("examples/assets/sample_1280_720.mp4"))
        }
        TestSample::Generic => None,
    }
}

pub fn get_asset_path(asset: TestSample) -> Result<PathBuf> {
    let path = map_asset_to_path(&asset).unwrap();
    match ensure_asset_available(&path) {
        Ok(()) => Ok(path),
        Err(e) => Err(e),
    }
}

fn ensure_asset_available(asset_path: &PathBuf) -> Result<()> {
    if !asset_path.exists() {
        return Err(anyhow!(
            "asset under path {:?} does not exist, try downloading it again",
            asset_path
        ));
    }
    Ok(())
}

fn examples_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

//to be deleted and substituted by path_download_file (needs refactoring all examples)
pub fn download_file(url: &str, path: &str) -> Result<PathBuf> {
    let sample_path = env::current_dir()?.join(path);
    fs::create_dir_all(sample_path.parent().unwrap())?;

    if sample_path.exists() {
        return Ok(sample_path);
    }

    let mut resp = reqwest::blocking::get(url)?;
    let mut out = File::create(sample_path.clone())?;
    io::copy(&mut resp, &mut out)?;
    Ok(sample_path)
}

fn download_asset(asset: &AssetData) -> Result<()> {
    fs::create_dir_all(asset.path.parent().unwrap())?;
    if !asset.path.exists() {
        let mut resp = reqwest::blocking::get(&asset.url)?;
        let mut out = File::create(asset.path.clone())?;
        io::copy(&mut resp, &mut out)?;
    }
    Ok(())
}
