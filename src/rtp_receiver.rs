use anyhow::Result;
use compositor_common::scene::InputId;
use compositor_pipeline::{
    error::CustomError,
    pipeline::{decoder::DecoderParameters, PipelineInput},
};
use crossbeam_channel::{bounded, Receiver, Sender};
use log::warn;
use std::{
    ffi::CString,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    ptr, thread,
};

use ffmpeg_next::{
    ffi::{
        avformat_alloc_context, avformat_close_input, avformat_find_stream_info,
        avformat_open_input,
    },
    format::context,
    media::Type,
    util::interrupt,
    Dictionary, Packet,
};

pub struct RtpReceiver {
    thread_finished: Receiver<()>,
    should_close: Sender<()>,
    decoder_parameters: DecoderParameters,
    pub(crate) port: u16,
}

pub struct Options {
    pub port: u16,
    pub input_id: InputId,
}

impl PipelineInput for RtpReceiver {
    type Opts = Options;
    type PacketIterator = crossbeam_channel::IntoIter<Packet>;

    fn new(opts: Self::Opts) -> Result<(Self, Self::PacketIterator), CustomError> {
        let (drop_sender, drop_receiver) = bounded(0);
        let (should_close_sender, should_close_receiver) = bounded(1);
        let (decoder_params_sender, decoder_params_receiver) = bounded(0);

        let port = opts.port;
        let (packet_sender, packet_receiver) = bounded(0);

        thread::spawn(move || {
            RtpReceiver::run(
                opts.port,
                should_close_receiver,
                packet_sender,
                decoder_params_sender,
            );
            drop_sender.send(())
        });

        Ok((
            Self {
                thread_finished: drop_receiver,
                should_close: should_close_sender,
                decoder_parameters: decoder_params_receiver
                    .recv()
                    .unwrap()
                    .map_err(CustomError)?,
                port,
            },
            packet_receiver.into_iter(),
        ))
    }

    fn decoder_parameters(&self) -> DecoderParameters {
        self.decoder_parameters
    }
}

impl Drop for RtpReceiver {
    fn drop(&mut self) {
        // - should_close signals to RTP thread that it should abort
        // - drop_receiver signals to drop method that RTP thread finished cleanup
        self.should_close.send(()).unwrap();
        self.thread_finished.recv().unwrap();
    }
}

struct ParamsWrapper(ffmpeg_next::codec::Parameters);
impl From<ParamsWrapper> for DecoderParameters {
    fn from(params: ParamsWrapper) -> Self {
        DecoderParameters {
            codec: match params.0.id() {
                ffmpeg_next::codec::Id::H264 => compositor_pipeline::pipeline::decoder::Codec::H264,
                _ => unimplemented!(),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error(transparent)]
    FfmpegError(#[from] ffmpeg_next::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl RtpReceiver {
    fn run(
        port: u16,
        should_close: Receiver<()>,
        packet_sender: Sender<Packet>,
        decoder_params_sender: Sender<
            Result<DecoderParameters, Box<dyn std::error::Error + Send + Sync + 'static>>,
        >,
    ) {
        let sdp_filepath = PathBuf::from(format!("/tmp/sdp_input_{}.sdp", port));
        let sdp_file_result = File::create(&sdp_filepath)
            .map_err(InitError::IoError)
            .and_then(|mut file| {
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
                )
                .map_err(InitError::IoError)
            });

        if let Err(e) = sdp_file_result {
            decoder_params_sender.send(Err(e.into())).unwrap();
            return;
        }

        // careful: moving the input context in any way will cause ffmpeg to segfault
        // I do not know why this happens
        let input_ctx = match input_with_dictionary_and_interrupt(
            &sdp_filepath,
            Dictionary::from_iter([("protocol_whitelist", "file,udp,rtp")]),
            || should_close.try_recv().is_ok(),
        ) {
            Ok(i) => i,
            Err(e) => {
                decoder_params_sender
                    .send(Err(Box::new(InitError::FfmpegError(e))))
                    .unwrap();
                return;
            }
        };

        let input = match input_ctx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)
        {
            Ok(input) => input,
            Err(e) => {
                decoder_params_sender.send(Err(e.into())).unwrap();
                return;
            }
        };

        let input_index = input.index();

        decoder_params_sender
            .send(Ok(ParamsWrapper(input.parameters()).into()))
            .unwrap();

        for packet in PacketIter::new(input_ctx, input_index) {
            packet_sender.send(packet).unwrap();
        }
    }
}

/// Combined implementation of ffmpeg_next::format:input_with_interrupt and
/// ffmpeg_next::format::input_with_dictionary that allows passing both interrupt
/// callback and Dictionary with options
pub fn input_with_dictionary_and_interrupt<P, F>(
    path: &P,
    options: Dictionary,
    closure: F,
) -> Result<context::Input, ffmpeg_next::Error>
where
    P: AsRef<Path>,
    F: FnMut() -> bool,
{
    fn from_path<P: AsRef<Path>>(path: &P) -> CString {
        CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap()
    }
    unsafe {
        let mut ps = avformat_alloc_context();

        (*ps).interrupt_callback = interrupt::new(Box::new(closure)).interrupt;

        let path = from_path(path);
        let mut opts = options.disown();
        let res = avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), &mut opts);

        Dictionary::own(opts);

        match res {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(ffmpeg_next::Error::from(e))
                }
            },

            e => Err(ffmpeg_next::Error::from(e)),
        }
    }
}

/// Implementation based on PacketIter from ffmpeg_next. Original code
/// was ignoring ffmpeg_next::Error::Exit, so it was not impossible
/// to stop RTP reader using interrupt callback.
pub struct PacketIter {
    context: context::Input,
    stream_index: usize,
}

impl PacketIter {
    pub fn new(context: context::Input, stream_index: usize) -> Self {
        PacketIter {
            context,
            stream_index,
        }
    }
}

impl Iterator for PacketIter {
    type Item = Packet;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut packet = Packet::empty();

        loop {
            match packet.read(&mut self.context) {
                Ok(..) => {
                    if packet.stream() != self.stream_index {
                        warn!("Received packet from unknown stream, skipping");
                        continue;
                    }

                    return Some(packet);
                }

                Err(ffmpeg_next::Error::Eof | ffmpeg_next::Error::Exit) => return None,
                Err(_) => (),
            }
        }
    }
}
