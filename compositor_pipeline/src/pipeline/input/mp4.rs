use std::{
    fs::File,
    io::Read,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use bytes::{Buf, Bytes, BytesMut};
use compositor_render::InputId;
use crossbeam_channel::Receiver;
use mp4::Mp4Reader;

use crate::pipeline::{
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
}

pub struct Mp4 {
    pub input_id: InputId,
    video_thread: Option<std::thread::JoinHandle<()>>,
    delete_file: bool,
    path: PathBuf,
    stop_video_reader: Arc<AtomicBool>,
}

impl Mp4 {
    pub fn new(options: Mp4Options) -> Result<(Self, ChunksReceiver), Mp4Error> {
        let (input_path, delete_file) = match options.source {
            Source::Url(url) => {
                let mut file_response = reqwest::blocking::get(url)?;

                let mut path = std::env::var("LIVE_COMPOSITOR_TMP_PATH")
                    .map(PathBuf::from)
                    .unwrap_or(std::env::temp_dir());

                path.push(format!(
                    "live-compositor-user-file-{}.mp4",
                    rand::random::<u64>()
                ));

                let mut file = std::fs::File::create(&path)?;

                std::io::copy(&mut file_response, &mut file)?;

                (path, true)
            }
            Source::File(path) => (path, false),
        };

        let input_file = std::fs::File::open(input_path.clone())?;
        let size = input_file.metadata()?.size();
        let reader = mp4::Mp4Reader::read_header(input_file, size)?;
        let stop_video_reader = Arc::new(AtomicBool::new(false));

        let (video_thread, video_receiver) =
            spawn_video_reader(reader, options.input_id.clone(), stop_video_reader.clone())?;

        Ok((
            Self {
                input_id: options.input_id,
                video_thread: Some(video_thread),
                delete_file,
                path: input_path,
                stop_video_reader,
            },
            ChunksReceiver {
                video: Some(video_receiver),
                audio: None,
            },
        ))
    }
}

impl Drop for Mp4 {
    fn drop(&mut self) {
        if let Some(thread) = self.video_thread.take() {
            self.stop_video_reader
                .store(true, std::sync::atomic::Ordering::Relaxed);
            thread.join().unwrap();
        }

        if self.delete_file {
            std::fs::remove_file(&self.path).unwrap();
        }
    }
}

fn spawn_video_reader(
    mut reader: Mp4Reader<File>,
    input_id: InputId,
    stop_thread: Arc<AtomicBool>,
) -> Result<(JoinHandle<()>, Receiver<EncodedChunk>), Mp4Error> {
    let (&track_id, track, avc1) = reader
        .tracks()
        .iter()
        .find_map(|(id, track)| {
            let track_type = match track.track_type() {
                Ok(t) => t,
                Err(_) => return None,
            };

            let media_type = match track.media_type() {
                Ok(t) => t,
                Err(_) => return None,
            };

            if track_type != mp4::TrackType::Video
                || media_type != mp4::MediaType::H264
                || track.trak.mdia.minf.stbl.stsd.avc1.is_none()
            {
                return None;
            }

            track
                .trak
                .mdia
                .minf
                .stbl
                .stsd
                .avc1
                .as_ref()
                .map(|avc1| (id, track, avc1))
        })
        .ok_or(Mp4Error::NoTrack)?;

    let (sender, receiver) = crossbeam_channel::bounded(10);

    // sps and pps have to be extracted from the container, interleaved with [0, 0, 0, 1],
    // concatenated and prepended to the first frame.
    let sps = avc1
        .avcc
        .sequence_parameter_sets
        .iter()
        .flat_map(|s| [0, 0, 0, 1].iter().chain(s.bytes.iter()));

    let pps = avc1
        .avcc
        .picture_parameter_sets
        .iter()
        .flat_map(|s| [0, 0, 0, 1].iter().chain(s.bytes.iter()));

    let sps_and_pps_payload = Some(sps.chain(pps).copied().collect::<Bytes>());

    let sample_count = track.sample_count();
    let timescale = track.timescale();
    let length_size = avc1.avcc.length_size_minus_one + 1;

    let thread = std::thread::Builder::new()
        .name(format!("mp4 reader {input_id}"))
        .spawn(move || {
            let mut sps_and_pps_payload = sps_and_pps_payload;
            let mut previous_sent_ts = None;
            for i in 1..sample_count {
                match reader.read_sample(track_id, i) {
                    Ok(Some(sample)) => {
                        let mut sample_data = sample.bytes.reader();
                        let mut data: BytesMut = Default::default();

                        if let Some(first_nal) = sps_and_pps_payload.take() {
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

                        let chunk = EncodedChunk {
                            data: data.freeze(),
                            pts: Duration::from_secs_f64(
                                (sample.start_time as f64 + sample.rendering_offset as f64)
                                    / timescale as f64,
                            ),
                            dts: Some(Duration::from_secs_f64(
                                sample.start_time as f64 / timescale as f64,
                            )),
                            kind: EncodedChunkKind::Video(VideoCodec::H264),
                        };

                        // TODO: this is a temporary soulution. the queue should decide when to take our buffers.
                        if let Some((send_time, previous_dts)) = previous_sent_ts {
                            let frame_difference = chunk.dts.unwrap().saturating_sub(previous_dts);
                            let should_be_sent: Instant = send_time + frame_difference;
                            let time_to_wait =
                                should_be_sent.saturating_duration_since(Instant::now());
                            std::thread::sleep(time_to_wait);
                        }

                        let chunk_dts = chunk.dts.unwrap();
                        let mut chunk = Some(chunk);
                        loop {
                            previous_sent_ts = Some((Instant::now(), chunk_dts));
                            match sender.send_timeout(chunk.take().unwrap(), Duration::from_millis(50)) {
                                Ok(()) => {
                                    break;
                                },
                                Err(crossbeam_channel::SendTimeoutError::Timeout(not_sent_chunk)) => {
                                    chunk = Some(not_sent_chunk);
                                },
                                Err(crossbeam_channel::SendTimeoutError::Disconnected(_)) => {
                                    log::error!("channel disconnected unexpectedly. Terminating.");
                                    return;
                                }
                            }

                            if stop_thread.load(std::sync::atomic::Ordering::Relaxed) {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Error while reading MP4 video sample: {:?}", e)
                    }
                    _ => {}
                }
            }
        })
        .unwrap();

    Ok((thread, receiver))
}
