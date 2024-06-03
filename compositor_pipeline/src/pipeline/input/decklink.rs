use std::sync::Arc;

use compositor_render::InputId;
use tracing::{error, span, Level};

use self::{capture::ChannelCallbackAdapter, find_device::find_decklink};

use super::{AudioInputReceiver, Input, InputInitInfo, InputInitResult, VideoInputReceiver};

mod capture;
mod find_device;

const AUDIO_SAMPLE_RATE: u32 = 48_000;

#[derive(Debug, thiserror::Error)]
pub enum DeckLinkError {
    #[error("Unknown DeckLink error.")]
    DecklinkError(#[from] decklink::DeckLinkError),
    #[error("No DeckLink device matches specified options.")]
    NoMatchingDeckLink,
    #[error("Selected device does not support capture.")]
    NoCaptureSupport,
    #[error("Selected device does not support input format detection.")]
    NoInputFormatDetection,
}

pub struct DeckLinkOptions {
    pub subdevice_index: Option<u32>,
    pub display_name: Option<String>,
    pub enable_audio: bool,
}

pub struct DeckLink {
    input: Arc<decklink::Input>,
}

impl DeckLink {
    pub(super) fn start_new_input(
        input_id: &InputId,
        opts: DeckLinkOptions,
    ) -> Result<InputInitResult, DeckLinkError> {
        let span = span!(
            Level::INFO,
            "DeckLink input",
            input_id = input_id.to_string()
        );
        let input = Arc::new(find_decklink(&opts)?.input()?);

        // Initial options, real config should be set based on detected format
        input.enable_video(
            decklink::DisplayModeType::ModeHD720p50,
            decklink::PixelFormat::Format8BitYUV,
            decklink::VideoInputFlags {
                enable_format_detection: true,
                ..Default::default()
            },
        )?;
        input.enable_audio(AUDIO_SAMPLE_RATE, decklink::AudioSampleType::Sample32bit, 2)?;

        let (callback, video_receiver, audio_receiver) = ChannelCallbackAdapter::new(
            span,
            opts.enable_audio,
            Arc::<decklink::Input>::downgrade(&input),
        );
        input.set_callback(Box::new(callback))?;
        input.start_streams()?;

        Ok(InputInitResult {
            input: Input::DeckLink(Self { input }),
            video: video_receiver.map(|rec| VideoInputReceiver::Raw {
                frame_receiver: rec,
            }),
            audio: audio_receiver.map(|rec| AudioInputReceiver::Raw {
                sample_receiver: rec,
                sample_rate: AUDIO_SAMPLE_RATE,
            }),
            init_info: InputInitInfo { port: None },
        })
    }
}

impl Drop for DeckLink {
    fn drop(&mut self) {
        if let Err(err) = self.input.stop_streams() {
            error!("Failed to stop streams: {:?}", err);
        }
    }
}
