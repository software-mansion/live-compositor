use std::{
    fs::File,
    io::Read,
    ops::ControlFlow,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use bytes::{Buf, Bytes, BytesMut};
use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use mp4::Mp4Reader;
use symphonia::core::formats::FormatReader;

use crate::pipeline::{
    decoder::{
        AacDecoderOptions, AacTransport, AudioDecoderOptions, DecoderOptions, VideoDecoderOptions,
    },
    structs::{EncodedChunk, EncodedChunkKind},
    VideoCodec,
};

use super::ChunksReceiver;

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

struct JoinableThread {
    join_handle: Option<std::thread::JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,
}

impl Drop for JoinableThread {
    fn drop(&mut self) {
        self.stop_signal
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let Some(handle) = self.join_handle.take() else {
            log::error!("Cannot join MP4 reader thread: no handle");
            return;
        };

        handle.join().unwrap();
    }
}

pub struct Mp4 {
    pub input_id: InputId,
    video_thread: Option<JoinableThread>,
    audio_thread: Option<JoinableThread>,
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

        let video_reader_info = spawn_video_reader(&input_path, options.input_id.clone())?;

        let audio_reader_info = spawn_audio_reader(&input_path, options.input_id.clone())?;

        Ok((
            Self {
                input_id: options.input_id,
                video_thread: video_reader_info.thread,
                audio_thread: audio_reader_info.thread,
                source: options.source,
                path_to_file: input_path,
            },
            ChunksReceiver {
                video: video_reader_info.receiver,
                audio: audio_reader_info.receiver,
            },
            DecoderOptions {
                video: video_reader_info.options,
                audio: audio_reader_info.options,
            },
        ))
    }
}

impl Drop for Mp4 {
    fn drop(&mut self) {
        drop(self.video_thread.take());
        drop(self.audio_thread.take());
        if let Source::Url(_) = self.source {
            std::fs::remove_file(&self.path_to_file).unwrap();
        }
    }
}

#[derive(Default)]
struct VideoReaderInfo {
    thread: Option<JoinableThread>,
    receiver: Option<Receiver<EncodedChunk>>,
    options: Option<VideoDecoderOptions>,
}

fn spawn_video_reader(input_path: &Path, input_id: InputId) -> Result<VideoReaderInfo, Mp4Error> {
    let stop_thread = Arc::new(AtomicBool::new(false));
    let stop_thread_clone = stop_thread.clone();
    let input_file = std::fs::File::open(input_path)?;
    let size = input_file.metadata()?.size();
    let reader = mp4::Mp4Reader::read_header(input_file, size)?;

    let Some((&track_id, track, avc)) = reader.tracks().iter().find_map(|(id, track)| {
        let track_type = track.track_type().ok()?;

        let media_type = track.media_type().ok()?;

        let avc = track.avc1_or_3_inner();

        if track_type != mp4::TrackType::Video
            || media_type != mp4::MediaType::H264
            || avc.is_none()
        {
            return None;
        }

        avc.map(|avc| (id, track, avc))
    }) else {
        return Ok(Default::default());
    };

    let (sender, receiver) = crossbeam_channel::bounded(10);

    // sps and pps have to be extracted from the container, interleaved with [0, 0, 0, 1],
    // concatenated and prepended to the first frame.
    let sps = avc
        .avcc
        .sequence_parameter_sets
        .iter()
        .flat_map(|s| [0, 0, 0, 1].iter().chain(s.bytes.iter()));

    let pps = avc
        .avcc
        .picture_parameter_sets
        .iter()
        .flat_map(|s| [0, 0, 0, 1].iter().chain(s.bytes.iter()));

    let sps_and_pps_payload = Some(sps.chain(pps).copied().collect::<Bytes>());

    let sample_count = track.sample_count();
    let timescale = track.timescale();
    let length_size = avc.avcc.length_size_minus_one + 1;

    let thread = std::thread::Builder::new()
        .name(format!("mp4 video reader {input_id}"))
        .spawn(move || {
            run_video_thread(
                sps_and_pps_payload,
                reader,
                sender,
                stop_thread_clone,
                length_size,
                TrackInfo {
                    sample_count,
                    timescale,
                    track_id,
                },
            )
        })
        .unwrap();

    Ok(VideoReaderInfo {
        thread: Some(JoinableThread {
            join_handle: Some(thread),
            stop_signal: stop_thread,
        }),
        receiver: Some(receiver),
        options: Some(VideoDecoderOptions {
            codec: VideoCodec::H264,
        }),
    })
}

struct TrackInfo {
    sample_count: u32,
    track_id: u32,
    timescale: u32,
}

fn run_video_thread(
    mut sps_and_pps: Option<Bytes>,
    mut reader: Mp4Reader<File>,
    sender: Sender<EncodedChunk>,
    stop_thread: Arc<AtomicBool>,
    length_size: u8,
    TrackInfo {
        sample_count,
        track_id,
        timescale,
    }: TrackInfo,
) {
    for i in 1..sample_count {
        match reader.read_sample(track_id, i) {
            Ok(Some(sample)) => {
                let mut sample_data = sample.bytes.reader();
                let mut data: BytesMut = Default::default();

                if let Some(first_nal) = sps_and_pps.take() {
                    data.extend_from_slice(&first_nal);
                }

                // the mp4 sample contains one h264 access unit (possibly more than one NAL).
                // the NALs are stored as: <length_size bytes long big endian encoded length><the NAL>.
                // we need to convert this into Annex B, in which NALs are separated by
                // [0, 0, 0, 1]. `lenght_size` is at most 4 bytes long.
                loop {
                    let mut len = [0u8; 4];

                    if sample_data
                        .read_exact(&mut len[4 - length_size as usize..])
                        .is_err()
                    {
                        break;
                    }

                    let len = u32::from_be_bytes(len);

                    let mut nalu = bytes::BytesMut::zeroed(len as usize);
                    sample_data.read_exact(&mut nalu).unwrap();

                    data.extend_from_slice(&[0, 0, 0, 1]);
                    data.extend_from_slice(&nalu);
                }

                let dts = Duration::from_secs_f64(sample.start_time as f64 / timescale as f64);
                let chunk = EncodedChunk {
                    data: data.freeze(),
                    pts: Duration::from_secs_f64(
                        (sample.start_time as f64 + sample.rendering_offset as f64)
                            / timescale as f64,
                    ),
                    dts: Some(dts),
                    kind: EncodedChunkKind::Video(VideoCodec::H264),
                };

                if let ControlFlow::Break(_) = send_chunk(chunk, &sender, &stop_thread) {
                    break;
                }
            }
            Err(e) => {
                log::warn!("Error while reading MP4 video sample: {:?}", e)
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct AudioReaderInfo {
    thread: Option<JoinableThread>,
    receiver: Option<Receiver<EncodedChunk>>,
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

    let handle = std::thread::Builder::new()
        .name(format!("mp4 audio reader {input_id}"))
        .spawn(move || run_audio_thread(cloned_track, reader, sender, stop_thread_clone))
        .unwrap();

    Ok(AudioReaderInfo {
        thread: Some(JoinableThread {
            join_handle: Some(handle),
            stop_signal: stop_thread,
        }),
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
    sender: Sender<EncodedChunk>,
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

        if let ControlFlow::Break(_) = send_chunk(chunk, &sender, &stop_thread) {
            break;
        }
    }
}

fn send_chunk(
    chunk: EncodedChunk,
    sender: &Sender<EncodedChunk>,
    stop_thread: &AtomicBool,
) -> ControlFlow<(), ()> {
    let mut chunk = Some(chunk);
    loop {
        match sender.send_timeout(chunk.take().unwrap(), Duration::from_millis(50)) {
            Ok(()) => {
                return ControlFlow::Continue(());
            }
            Err(crossbeam_channel::SendTimeoutError::Timeout(not_sent_chunk)) => {
                chunk = Some(not_sent_chunk);
            }
            Err(crossbeam_channel::SendTimeoutError::Disconnected(_)) => {
                log::error!("channel disconnected unexpectedly. Terminating.");
                return ControlFlow::Break(());
            }
        }

        if stop_thread.load(std::sync::atomic::Ordering::Relaxed) {
            return ControlFlow::Break(());
        }
    }
}
