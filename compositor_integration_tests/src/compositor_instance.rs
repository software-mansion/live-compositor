use crate::CommunicationProtocol;
use anyhow::Result;
use compositor_render::use_global_wgpu_ctx;
use reqwest::StatusCode;
use std::{
    env,
    io::Write,
    net::{Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};
use video_compositor::{http, logger};

pub struct CompositorInstance {
    pub api_port: u16,
    pub http_client: reqwest::blocking::Client,
}

impl CompositorInstance {
    pub fn start(api_port: u16) -> Self {
        env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
        ffmpeg_next::format::network::init();
        logger::init_logger();

        use_global_wgpu_ctx();

        thread::Builder::new()
            .name("compositor instance on port".to_owned())
            .spawn(move || {
                http::Server::new(api_port).run();
            })
            .unwrap();

        CompositorInstance {
            api_port,
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn send_request(&mut self, request_body: serde_json::Value) -> Result<()> {
        let resp = self
            .http_client
            .post(format!("http://127.0.0.1:{}/--/api", self.api_port))
            .timeout(Duration::from_secs(100))
            .json(&request_body)
            .send()?;

        if resp.status() >= StatusCode::BAD_REQUEST {
            let status = resp.status();
            let request_str = serde_json::to_string_pretty(&request_body).unwrap();
            let body_str = resp.text().unwrap();
            return Err(anyhow::anyhow!(
                "Request failed with status: {status}\nRequest: {request_str}\nResponse: {body_str}",
            ));
        }

        Ok(())
    }
}

pub struct PacketSender {
    protocol: CommunicationProtocol,
    socket: socket2::Socket,
}

impl PacketSender {
    pub fn new(protocol: CommunicationProtocol, port: u16) -> Result<Self> {
        let socket = match protocol {
            CommunicationProtocol::Udp => socket2::Socket::new(
                socket2::Domain::IPV4,
                socket2::Type::DGRAM,
                Some(socket2::Protocol::UDP),
            )?,
            CommunicationProtocol::Tcp => socket2::Socket::new(
                socket2::Domain::IPV4,
                socket2::Type::STREAM,
                Some(socket2::Protocol::TCP),
            )?,
        };

        socket.connect(&SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port).into())?;

        Ok(Self { protocol, socket })
    }

    pub fn send(&mut self, rtp_packets: &[u8]) -> Result<()> {
        match self.protocol {
            CommunicationProtocol::Udp => self.send_via_udp(rtp_packets),
            CommunicationProtocol::Tcp => self.send_via_tcp(rtp_packets),
        }
    }

    fn send_via_udp(&mut self, rtp_packets: &[u8]) -> Result<()> {
        let mut sent_bytes = 0;
        while sent_bytes < rtp_packets.len() {
            let packet_len =
                u16::from_be_bytes([rtp_packets[sent_bytes], rtp_packets[sent_bytes + 1]]) as usize;
            sent_bytes += 2;

            let packet = &rtp_packets[sent_bytes..(sent_bytes + packet_len)];

            sent_bytes += packet_len;

            self.socket.write_all(packet)?;
        }

        Ok(())
    }

    fn send_via_tcp(&mut self, rtp_packets: &[u8]) -> Result<()> {
        self.socket.write_all(rtp_packets)?;
        Ok(())
    }
}
