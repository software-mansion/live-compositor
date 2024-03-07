use std::{
    ops::ControlFlow,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use compositor_render::InputId;
use crossbeam_channel::{Receiver, SendTimeoutError, Sender};
use symphonia::core::formats::FormatReader;
use tracing::{debug, error, span, Level};

use crate::{
    pipeline::{
        decoder::{AacDecoderOptions, AacTransport, AudioDecoderOptions, DecoderOptions},
        structs::{EncodedChunk, EncodedChunkKind},
    },
    queue::PipelineEvent,
};

use self::video_reader::{Mp4ReaderOptions, VideoReader};

use super::ChunksReceiver;

pub mod video_reader;

pub struct Mp4Options {
    pub source: Source,
    pub input_id: InputId,
}

pub enum Source {
    Url(String),
    File(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum Mp4Error {
    #[error("Error while doing file operations.")]
    IoError(#[from] std::io::Error),

    #[error("Error while downloading the MP4 file.")]
    HttpError(#[from] reqwest::Error),

    #[error("Mp4 reader error.")]
    Mp4ReaderError(#[from] mp4::Error),

    #[error("No suitable track in the mp4 file")]
    NoTrack,

    #[error("Symphonia mp4 reader error.")]
    SymphoniaError(#[from] symphonia::core::errors::Error),
}

pub struct Mp4 {
    pub input_id: InputId,
    _video_thread: Option<VideoReader>,
    stop_audio_thread: Option<Arc<AtomicBool>>,
    source: Source,
    path_to_file: PathBuf,
}

impl Mp4 {
    pub fn new(
        options: Mp4Options,
        download_dir: &Path,
    ) -> Result<(Self, ChunksReceiver, DecoderOptions), Mp4Error> {
        let input_path = match options.source {
            Source::Url(ref url) => {
                let file_response = reqwest::blocking::get(url)?;
                let mut file_response = file_response.error_for_status()?;

                let mut path = download_dir.to_owned();
                path.push(format!(
                    "live-compositor-user-file-{}.mp4",
                    rand::random::<u64>()
                ));

                let mut file = std::fs::File::create(&path)?;

                std::io::copy(&mut file_response, &mut file)?;

                path
            }
            Source::File(ref path) => path.clone(),
        };

        let (video_reader, video_receiver, video_decoder_options) = match VideoReader::new(
            Mp4ReaderOptions::NonFragmented {
                file: input_path.clone(),
            },
            options.input_id.clone(),
        )? {
            Some((reader, receiver)) => {
                let decoder_options = reader.decoder_options();
                (Some(reader), Some(receiver), Some(decoder_options))
            }
            None => (None, None, None),
        };

        let audio_reader_info = spawn_audio_reader(&input_path, options.input_id.clone())?;

        Ok((
            Self {
                input_id: options.input_id,
                _video_thread: video_reader,
                stop_audio_thread: audio_reader_info.stop_thread,
                source: options.source,
                path_to_file: input_path,
            },
            ChunksReceiver {
                video: video_receiver,
                audio: audio_reader_info.receiver,
            },
            DecoderOptions {
                video: video_decoder_options,
                audio: audio_reader_info.options,
            },
        ))
    }
}

impl Drop for Mp4 {
    fn drop(&mut self) {
        if let Some(stop_audio) = &self.stop_audio_thread {
            stop_audio.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        if let Source::Url(_) = self.source {
            if let Err(e) = std::fs::remove_file(&self.path_to_file) {
                error!(input_id=?self.input_id.0, "Error while removing the downloaded mp4 file: {e}");
            }
        }
    }
}

#[derive(Default)]
struct AudioReaderInfo {
    stop_thread: Option<Arc<AtomicBool>>,
    receiver: Option<Receiver<PipelineEvent<EncodedChunk>>>,
    options: Option<AudioDecoderOptions>,
}

fn spawn_audio_reader(input_path: &Path, input_id: InputId) -> Result<AudioReaderInfo, Mp4Error> {
    let stop_thread = Arc::new(AtomicBool::new(false));
    let stop_thread_clone = stop_thread.clone();
    let input_file = std::fs::File::open(input_path)?;
    let source =
        symphonia::core::io::MediaSourceStream::new(Box::new(input_file), Default::default());
    let mut reader =
        symphonia::default::formats::IsoMp4Reader::try_new(source, &Default::default())?;

    let track = reader
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec == symphonia::core::codecs::CODEC_TYPE_AAC)
        .cloned();

    let Some(track) = track else {
        return Ok(Default::default());
    };

    reader.seek(
        symphonia::core::formats::SeekMode::Coarse,
        symphonia::core::formats::SeekTo::Time {
            time: track.codec_params.start_ts.into(),
            track_id: Some(track.id),
        },
    )?;

    let first_packet = loop {
        let packet = reader.next_packet()?;

        if packet.track_id() == track.id {
            reader.seek(
                symphonia::core::formats::SeekMode::Coarse,
                symphonia::core::formats::SeekTo::Time {
                    time: track.codec_params.start_ts.into(),
                    track_id: Some(track.id),
                },
            )?;
            break packet;
        }
    };

    let transport = if first_packet.data[..4] == [b'A', b'D', b'I', b'F'] {
        AacTransport::ADIF
    } else if first_packet.data[0] == 0xff && first_packet.data[1] & 0xf0 == 0xf0 {
        AacTransport::ADTS
    } else {
        AacTransport::RawAac
    };

    let (sender, receiver) = crossbeam_channel::bounded(50);
    let cloned_track = track.clone();

    std::thread::Builder::new()
        .name(format!("mp4 audio reader {input_id}"))
        .spawn(move || {
            let _span = span!(Level::INFO, "MP4 audio", input_id = input_id.to_string()).entered();
            run_audio_thread(cloned_track, reader, sender, stop_thread_clone);
            debug!("Closing MP4 audio reader thread");
        })
        .unwrap();

    Ok(AudioReaderInfo {
        stop_thread: Some(stop_thread),
        receiver: Some(receiver),
        options: Some(AudioDecoderOptions::Aac(AacDecoderOptions {
            transport,
            asc: track.codec_params.extra_data.map(|t| t.into()),
        })),
    })
}

fn run_audio_thread(
    track: symphonia::core::formats::Track,
    mut reader: symphonia::default::formats::IsoMp4Reader,
    sender: Sender<PipelineEvent<EncodedChunk>>,
    stop_thread: Arc<AtomicBool>,
) {
    while let Ok(packet) = reader.next_packet() {
        if packet.track_id() != track.id {
            continue;
        }

        let timebase = track
            .codec_params
            .time_base
            .unwrap_or(symphonia::core::units::TimeBase {
                numer: 1,
                denom: 48000,
            });

        let pts = std::time::Duration::from_secs_f64(
            packet.ts() as f64 * timebase.numer as f64 / timebase.denom as f64,
        );

        let chunk = EncodedChunk {
            data: packet.data.into(),
            pts,
            dts: None,
            kind: EncodedChunkKind::Audio(crate::pipeline::AudioCodec::Aac),
        };

        if let ControlFlow::Break(_) = send_chunk(PipelineEvent::Data(chunk), &sender, &stop_thread)
        {
            break;
        }
    }
    if let Err(_err) = sender.send(PipelineEvent::EOS) {
        debug!("Failed to send EOS from MP4 audio reader. Channel closed.");
    }
}

fn send_chunk(
    chunk: PipelineEvent<EncodedChunk>,
    sender: &Sender<PipelineEvent<EncodedChunk>>,
    stop_thread: &AtomicBool,
) -> ControlFlow<(), ()> {
    let mut chunk = Some(chunk);
    loop {
        match sender.send_timeout(chunk.take().unwrap(), Duration::from_millis(50)) {
            Ok(()) => {
                return ControlFlow::Continue(());
            }
            Err(SendTimeoutError::Timeout(not_sent_chunk)) => {
                chunk = Some(not_sent_chunk);
            }
            Err(SendTimeoutError::Disconnected(_)) => {
                debug!("Channel disconnected.");
                return ControlFlow::Break(());
            }
        }

        if stop_thread.load(std::sync::atomic::Ordering::Relaxed) {
            return ControlFlow::Break(());
        }
    }
}
