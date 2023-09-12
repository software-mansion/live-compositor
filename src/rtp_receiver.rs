use anyhow::{anyhow, Result};
use compositor_common::{
    error::ErrorStack,
    frame::YuvData,
    scene::{InputId, Resolution},
    Frame,
};
use compositor_pipeline::{pipeline::PipelineInput, queue::Queue};
use crossbeam_channel::{bounded, Receiver};
use log::{error, warn};
use std::{
    ffi::CString,
    fs::File,
    io::Write,
    mem,
    path::{Path, PathBuf},
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use ffmpeg_next::{
    codec::{self, Context, Id},
    ffi::{
        avformat_alloc_context, avformat_close_input, avformat_find_stream_info,
        avformat_open_input,
    },
    format::{self, context},
    frame::{self, Video},
    media::Type,
    util::interrupt,
    Dictionary, Packet, Stream,
};

pub struct RtpReceiver {
    drop_receiver: Receiver<()>,
    should_close: Arc<AtomicBool>,
    pub(crate) port: u16,
}

pub struct Options {
    pub port: u16,
    pub input_id: InputId,
}

impl PipelineInput for RtpReceiver {
    type Opts = Options;

    fn new(queue: Arc<Queue>, opts: Self::Opts) -> Self {
        let (drop_sender, drop_receiver) = bounded(0);
        let should_close = Arc::new(AtomicBool::new(false));
        let should_close_clone = should_close.clone();
        let port = opts.port;
        thread::spawn(move || {
            RtpReceiver::start(queue, opts.port, opts.input_id, should_close_clone).unwrap();
            drop_sender.send(())
        });
        Self {
            drop_receiver,
            should_close,
            port,
        }
    }
}

impl Drop for RtpReceiver {
    fn drop(&mut self) {
        // - AtomicBool signals to RTP thread that it should abort
        // - Channel signals to drop method that RTP thread finished cleanup
        self.should_close.store(true, Ordering::Relaxed);
        self.drop_receiver.recv().unwrap();
    }
}

impl RtpReceiver {
    fn start(
        queue: Arc<Queue>,
        port: u16,
        input_id: InputId,
        should_close: Arc<AtomicBool>,
    ) -> Result<()> {
        let sdp_filepath = PathBuf::from(format!("/tmp/sdp_input_{}.sdp", port));
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
        let mut input_ctx = input_with_dictionary_and_interrupt(
            &sdp_filepath,
            Dictionary::from_iter([("protocol_whitelist", "file,udp,rtp")]),
            || should_close.load(Ordering::Relaxed),
        )?;

        let input = input_ctx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let input_index = input.index();
        let decoder = Context::from_parameters(input.parameters())?;
        let decoder = decoder.decoder();
        let h264_codec = codec::decoder::find(Id::H264).unwrap();
        let mut decoder = decoder.open_as(h264_codec)?;

        let mut pts_offset: Option<i64> = None;
        let mut decoded_frame = frame::Video::empty();
        for (stream, packet) in PacketIter::new(&mut input_ctx) {
            if stream.index() != input_index {
                warn!("Received frame from unknown stream, skipping");
                continue;
            }

            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                let frame = match frame_from_av(&mut decoded_frame, &mut pts_offset) {
                    Ok(frame) => frame,
                    Err(err) => {
                        warn!("Dropping frame: {err}");
                        continue;
                    }
                };
                if let Err(err) = queue.enqueue_frame(input_id.clone(), frame) {
                    error!(
                        "Failed to push frame: {}",
                        ErrorStack::new(&err).into_string()
                    );
                }
            }
        }
        Ok(())
    }
}

fn frame_from_av(decoded: &mut Video, pts_offset: &mut Option<i64>) -> Result<Frame> {
    if decoded.format() != format::pixel::Pixel::YUV420P {
        return Err(anyhow!("only YUV420P is supported"));
    }
    let original_pts = decoded.pts();
    if let (Some(pts), None) = (decoded.pts(), &pts_offset) {
        *pts_offset = Some(-pts)
    }
    let pts = original_pts
        .map(|original_pts| original_pts + pts_offset.unwrap_or(0))
        .ok_or_else(|| anyhow!("missing pts"))?;
    let pts = Duration::from_secs_f64(f64::max((pts as f64) / 90000.0, 0.0));
    Ok(Frame {
        data: YuvData {
            y_plane: copy_plane_from_av(decoded, 0),
            u_plane: copy_plane_from_av(decoded, 1),
            v_plane: copy_plane_from_av(decoded, 2),
        },
        resolution: Resolution {
            width: decoded.width().try_into()?,
            height: decoded.height().try_into()?,
        },
        pts,
    })
}

fn copy_plane_from_av(decoded: &Video, plane: usize) -> bytes::Bytes {
    let mut output_buffer = bytes::BytesMut::with_capacity(
        decoded.plane_width(plane) as usize * decoded.plane_height(plane) as usize,
    );

    decoded
        .data(plane)
        .chunks(decoded.stride(plane))
        .map(|chunk| &chunk[..decoded.plane_width(plane) as usize])
        .for_each(|chunk| output_buffer.extend_from_slice(chunk));

    output_buffer.freeze()
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
pub struct PacketIter<'a> {
    context: &'a mut context::Input,
}

impl<'a> PacketIter<'a> {
    pub fn new(context: &mut context::Input) -> PacketIter {
        PacketIter { context }
    }
}

impl<'a> Iterator for PacketIter<'a> {
    type Item = (Stream<'a>, Packet);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut packet = Packet::empty();

        loop {
            match packet.read(self.context) {
                Ok(..) => unsafe {
                    return Some((
                        Stream::wrap(mem::transmute_copy(&self.context), packet.stream()),
                        packet,
                    ));
                },

                Err(ffmpeg_next::Error::Eof | ffmpeg_next::Error::Exit) => return None,
                Err(_) => (),
            }
        }
    }
}
