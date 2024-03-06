use std::{
    io::{Read, Seek},
    ops::ControlFlow,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use bytes::{Buf, Bytes, BytesMut};
use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use mp4::Mp4Reader;
use tracing::{debug, warn};

use crate::{
    pipeline::{
        decoder::VideoDecoderOptions,
        structs::{EncodedChunk, EncodedChunkKind},
        VideoCodec,
    },
    queue::PipelineEvent,
};

use super::Mp4Error;

type ChunkReceiver = Receiver<PipelineEvent<EncodedChunk>>;

pub struct VideoReader {
    stop_thread: Arc<AtomicBool>,
    fragment_sender: Option<Sender<PipelineEvent<Bytes>>>,
    decoder_options: VideoDecoderOptions,
}

pub enum Mp4ReaderOptions {
    NonFragmented {
        file: PathBuf,
    },
    Fragmented {
        header: Bytes,
        fragment_receiver: Receiver<PipelineEvent<Bytes>>,
    },
}

impl VideoReader {
    pub fn new(
        options: Mp4ReaderOptions,
        input_id: InputId,
    ) -> Result<Option<(Self, ChunkReceiver)>, Mp4Error> {
        let stop_thread = Arc::new(AtomicBool::new(false));

        match options {
            Mp4ReaderOptions::NonFragmented { file } => {
                let input_file = std::fs::File::open(file)?;
                let size = input_file.metadata()?.size();
                Self::new_helper(input_file, size, None, stop_thread, input_id)
            }
            Mp4ReaderOptions::Fragmented {
                header,
                fragment_receiver,
            } => {
                let size = header.len() as u64;
                let reader = std::io::Cursor::new(header);
                Self::new_helper(reader, size, Some(fragment_receiver), stop_thread, input_id)
            }
        }
    }

    /// necessary for the generic reader
    fn new_helper<R: Read + Seek + Send + 'static>(
        reader: R,
        size: u64,
        fragment_receiver: Option<Receiver<PipelineEvent<Bytes>>>,
        stop_thread: Arc<AtomicBool>,
        input_id: InputId,
    ) -> Result<Option<(Self, ChunkReceiver)>, Mp4Error> {
        let reader = mp4::Mp4Reader::read_header(reader, size)?;

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

        let stop_thread_clone = stop_thread.clone();

        std::thread::Builder::new()
            .name(format!("mp4 video reader {input_id}"))
            .spawn(move || {
                run_video_thread(
                    sps_and_pps_payload,
                    reader,
                    sender,
                    stop_thread_clone,
                    length_size,
                    fragment_receiver,
                    TrackInfo {
                        sample_count,
                        timescale,
                        track_id,
                    },
                    input_id.clone(),
                );
                debug!(input_id=?input_id.0, "Closing MP4 video reader thread");
            })
            .unwrap();

        Ok(Some((
            VideoReader {
                stop_thread,
                fragment_sender: None,
                decoder_options: VideoDecoderOptions {
                    codec: VideoCodec::H264,
                },
            },
            receiver,
        )))
    }

    pub fn decoder_options(&self) -> VideoDecoderOptions {
        self.decoder_options.clone()
    }

    pub fn fragment_sender(&self) -> Option<Sender<PipelineEvent<Bytes>>> {
        self.fragment_sender.clone()
    }
}

impl Drop for VideoReader {
    fn drop(&mut self) {
        self.stop_thread
            .store(true, std::sync::atomic::Ordering::Relaxed)
    }
}

struct TrackInfo {
    sample_count: u32,
    track_id: u32,
    timescale: u32,
}

#[allow(clippy::too_many_arguments)]
fn run_video_thread<R: Read + Seek>(
    mut sps_and_pps: Option<Bytes>,
    mut reader: Mp4Reader<R>,
    sender: Sender<PipelineEvent<EncodedChunk>>,
    stop_thread: Arc<AtomicBool>,
    length_size: u8,
    _fragment_receiver: Option<Receiver<PipelineEvent<Bytes>>>,
    TrackInfo {
        sample_count,
        track_id,
        timescale,
    }: TrackInfo,
    input_id: InputId,
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

                if let ControlFlow::Break(_) =
                    super::send_chunk(PipelineEvent::Data(chunk), &sender, &stop_thread, &input_id)
                {
                    break;
                }
            }
            Err(e) => {
                warn!(input_id=?input_id.0, "Error while reading MP4 video sample: {:?}", e);
            }
            _ => {}
        }
    }
    let _ = sender.send(PipelineEvent::EOS);
}
