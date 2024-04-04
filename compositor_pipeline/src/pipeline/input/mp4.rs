use std::path::{Path, PathBuf};

use bytes::Bytes;
use compositor_render::InputId;
use crossbeam_channel::Receiver;
use tracing::error;

use crate::{
    pipeline::decoder::{AudioDecoderOptions, DecoderOptions, VideoDecoderOptions},
    queue::PipelineEvent,
};

use mp4_file_reader::Mp4FileReader;

use super::ChunksReceiver;

pub mod mp4_file_reader;

pub struct Mp4Options {
    pub source: Source,
}

pub(crate) enum Mp4ReaderOptions {
    NonFragmented {
        file: PathBuf,
    },
    #[allow(dead_code)]
    Fragmented {
        header: Bytes,
        fragment_receiver: Receiver<PipelineEvent<Bytes>>,
    },
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
    _video_thread: Option<Mp4FileReader<VideoDecoderOptions>>,
    _audio_thread: Option<Mp4FileReader<AudioDecoderOptions>>,
    source: Source,
    path_to_file: PathBuf,
}

impl Mp4 {
    pub(crate) fn new(
        input_id: &InputId,
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

        let video = Mp4FileReader::new_video(
            Mp4ReaderOptions::NonFragmented {
                file: input_path.clone(),
            },
            input_id.clone(),
        )?;

        let (video_reader, video_receiver, video_decoder_options) = match video {
            Some((reader, receiver)) => {
                let decoder_options = reader.decoder_options();
                (Some(reader), Some(receiver), Some(decoder_options))
            }
            None => (None, None, None),
        };

        let audio = Mp4FileReader::new_audio(
            Mp4ReaderOptions::NonFragmented {
                file: input_path.clone(),
            },
            input_id.clone(),
        )?;

        let (audio_reader, audio_receiver, audio_deocder_options) = match audio {
            Some((reader, receiver)) => {
                let decoder_options = reader.decoder_options();
                (Some(reader), Some(receiver), Some(decoder_options))
            }
            None => (None, None, None),
        };

        Ok((
            Self {
                input_id: input_id.clone(),
                _video_thread: video_reader,
                _audio_thread: audio_reader,
                source: options.source,
                path_to_file: input_path,
            },
            ChunksReceiver {
                video: video_receiver,
                audio: audio_receiver,
            },
            DecoderOptions {
                video: video_decoder_options,
                audio: audio_deocder_options,
            },
        ))
    }
}

impl Drop for Mp4 {
    fn drop(&mut self) {
        if let Source::Url(_) = self.source {
            if let Err(e) = std::fs::remove_file(&self.path_to_file) {
                error!(input_id=?self.input_id.0, "Error while removing the downloaded mp4 file: {e}");
            }
        }
    }
}
