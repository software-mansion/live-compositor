use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, OnceLock, Weak,
    },
    time::{Duration, Instant},
};

use bytes::{BufMut, BytesMut};
use compositor_render::{error::ErrorStack, Frame, Resolution, YuvData, YuvVariant};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use decklink::{
    AudioInputPacket, DetectedVideoInputFormatFlags, DisplayMode, InputCallback,
    InputCallbackResult, PixelFormat, VideoInputFlags, VideoInputFormatChangedEvents,
    VideoInputFrame,
};
use tracing::{debug, info, trace, warn, Span};

use crate::{
    pipeline::structs::{DecodedSamples, Samples},
    queue::PipelineEvent,
};

use super::AUDIO_SAMPLE_RATE;

pub(super) struct ChannelCallbackAdapter {
    video_sender: Option<Sender<PipelineEvent<Frame>>>,
    audio_sender: Option<Sender<PipelineEvent<DecodedSamples>>>,
    span: Span,

    // I'm not sure, but I suspect that holding Arc here would create a circular
    // dependency
    input: Weak<decklink::Input>,
    start_time: OnceLock<Instant>,
    offset: Mutex<Duration>,

    // TODO: temporary (CPU processing might be to slow, so drop every second frame)
    counter: AtomicU64,
}

impl ChannelCallbackAdapter {
    pub(super) fn new(
        span: Span,
        enable_audio: bool,
        input: Weak<decklink::Input>,
    ) -> (
        Self,
        Option<Receiver<PipelineEvent<Frame>>>,
        Option<Receiver<PipelineEvent<DecodedSamples>>>,
    ) {
        let (video_sender, video_receiver) = bounded(1000);
        let (audio_sender, audio_receiver) = match enable_audio {
            true => {
                let (sender, receiver) = bounded(1000);
                (Some(sender), Some(receiver))
            }
            false => (None, None),
        };
        (
            Self {
                video_sender: Some(video_sender),
                audio_sender,
                span,
                input,
                start_time: OnceLock::new(),
                offset: Mutex::new(Duration::ZERO),
                counter: AtomicU64::new(0),
            },
            Some(video_receiver),
            audio_receiver,
        )
    }

    fn start_time(&self) -> Instant {
        *self.start_time.get_or_init(Instant::now)
    }

    fn handle_video_frame(
        &self,
        video_frame: &mut VideoInputFrame,
        sender: &Sender<PipelineEvent<Frame>>,
    ) -> Result<(), decklink::DeckLinkError> {
        // TODO: remove
        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        if count % 3 != 0 {
            return Ok(());
        }

        let offset = *self.offset.lock().unwrap();
        let pts = video_frame.stream_time()? + offset;

        let width = video_frame.width();
        let height = video_frame.height();
        let data = video_frame.bytes()?;
        let bytes_per_row = video_frame.bytes_per_row();
        let pixel_format = video_frame.pixel_format()?;

        let frame = match pixel_format {
            PixelFormat::Format8BitYUV => {
                Self::frame_from_yuv_422(width, height, bytes_per_row, data, pts)
            }
            // TODO just for testing
            PixelFormat::Format10BitRGB => {
                warn!(?pixel_format, "Unsupported pixel format");
                Self::frame_from_yuv_422(width, height, bytes_per_row, data, pts)
            }
            pixel_format => {
                warn!(?pixel_format, "Unsupported pixel format");
                return Ok(());
            }
        };

        trace!(?frame, ?pixel_format, "Received frame from decklink");
        match sender.try_send(PipelineEvent::Data(frame)) {
            Ok(_) => (),
            Err(TrySendError::Full(_)) => {
                warn!(
                    "Failed to send frame from DeckLink. Channel is full, dropping frame pts={pts:?}."
                )
            }
            Err(TrySendError::Disconnected(_)) => {
                debug!("Failed to send frame from DeckLink. Channel closed.");
            }
        }
        Ok(())
    }

    fn frame_from_yuv_422(
        width: usize,
        height: usize,
        bytes_per_row: usize,
        data: bytes::Bytes,
        pts: Duration,
    ) -> Frame {
        let mut y_plane = BytesMut::with_capacity(width * height);
        let mut u_plane = BytesMut::with_capacity((width / 2) * (height / 2));
        let mut v_plane = BytesMut::with_capacity((width / 2) * (height / 2));

        // TODO: temporary conversion in Rust. Rework it to do it on a GPU
        for (row_index, row) in data.chunks(bytes_per_row).enumerate() {
            for (index, pixel) in row.chunks(2).enumerate() {
                if index < width && pixel.len() >= 2 {
                    y_plane.put_u8(pixel[1]);
                }
            }
            if row_index % 2 == 0 && row_index < height {
                for (index, pixel) in row.chunks(4).enumerate() {
                    if index * 2 < width && pixel.len() >= 4 {
                        u_plane.put_u8(pixel[0]);
                        v_plane.put_u8(pixel[2]);
                    }
                }
            }
        }

        Frame {
            data: YuvData {
                variant: YuvVariant::YUV420P,
                y_plane: y_plane.freeze(),
                u_plane: u_plane.freeze(),
                v_plane: v_plane.freeze(),
            },
            resolution: Resolution { width, height },
            pts,
        }
    }

    fn handle_audio_packet(
        &self,
        audio_packet: &mut AudioInputPacket,
        sender: &Sender<PipelineEvent<DecodedSamples>>,
    ) -> Result<(), decklink::DeckLinkError> {
        let offset = *self.offset.lock().unwrap();
        let pts = audio_packet.packet_time()? + offset;

        let samples = audio_packet.as_32_bit_stereo()?;
        let samples = DecodedSamples {
            samples: Arc::new(Samples::Stereo32Bit(samples)),
            start_pts: pts,
            sample_rate: AUDIO_SAMPLE_RATE,
        };

        trace!(?samples, "Received audio samples from decklink");
        match sender.try_send(PipelineEvent::Data(samples)) {
            Ok(_) => (),
            Err(TrySendError::Full(_)) => {
                warn!(
                    "Failed to send samples from DeckLink. Channel is full, dropping samples pts={pts:?}."
                )
            }
            Err(TrySendError::Disconnected(_)) => {
                debug!("Failed to send samples from DeckLink. Channel closed.")
            }
        }
        Ok(())
    }

    fn handle_format_change(
        &self,
        display_mode: DisplayMode,
        flags: DetectedVideoInputFormatFlags,
    ) -> Result<(), decklink::DeckLinkError> {
        let Some(input) = self.input.upgrade() else {
            return Ok(());
        };

        let mode = display_mode.display_mode_type()?;

        // TODO: mostly placeholder to pick sth
        // in case of unsupported format
        let pixel_format = if flags.format_y_cb_cr_422 {
            if flags.bit_depth_8 {
                PixelFormat::Format8BitYUV
            } else if flags.bit_depth_10 {
                PixelFormat::Format10BitYUV
            } else {
                warn!("Unknown format, falling back to 8-bit YUV");
                PixelFormat::Format8BitYUV
            }
        } else if flags.format_rgb_444 {
            if flags.bit_depth_8 {
                PixelFormat::Format8BitBGRA
            } else if flags.bit_depth_10 {
                PixelFormat::Format10BitRGB
            } else if flags.bit_depth_12 {
                PixelFormat::Format12BitRGB
            } else {
                warn!("Unknown format, falling back to 10-bit RGB");
                PixelFormat::Format10BitRGB
            }
        } else {
            warn!("Unknown format, skipping change");
            return Ok(());
        };

        info!("Detected new input format {mode:?} {pixel_format:?} {flags:?}");

        input.pause_streams()?;
        input.enable_video(
            mode,
            pixel_format,
            VideoInputFlags {
                enable_format_detection: true,
                ..Default::default()
            },
        )?;
        input.flush_streams()?;
        input.start_streams()?;

        *self.offset.lock().unwrap() = self.start_time().elapsed();

        Ok(())
    }
}

impl InputCallback for ChannelCallbackAdapter {
    fn video_input_frame_arrived(
        &self,
        video_frame: Option<&mut VideoInputFrame>,
        audio_packet: Option<&mut AudioInputPacket>,
    ) -> InputCallbackResult {
        let _span = self.span.enter();

        // ensure init start time on first frame
        self.start_time();

        if let (Some(video_frame), Some(sender)) = (video_frame, &self.video_sender) {
            if let Err(err) = self.handle_video_frame(video_frame, sender) {
                warn!(
                    "Failed to handle video frame: {}",
                    ErrorStack::new(&err).into_string()
                )
            }
        }

        if let (Some(audio_packet), Some(sender)) = (audio_packet, &self.audio_sender) {
            if let Err(err) = self.handle_audio_packet(audio_packet, sender) {
                warn!(
                    "Failed to handle video frame: {}",
                    ErrorStack::new(&err).into_string()
                )
            }
        }

        InputCallbackResult::Ok
    }

    fn video_input_format_changed(
        &self,
        events: VideoInputFormatChangedEvents,
        display_mode: DisplayMode,
        flags: DetectedVideoInputFormatFlags,
    ) -> InputCallbackResult {
        let _span = self.span.enter();

        if events.field_dominance_changed
            || events.display_mode_changed
            || events.colorspace_changed
        {
            if let Err(err) = self.handle_format_change(display_mode, flags) {
                warn!(
                    "Failed to handle format change: {}",
                    ErrorStack::new(&err).into_string()
                );
            }
        }

        InputCallbackResult::Ok
    }
}
