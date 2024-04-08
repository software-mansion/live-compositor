use compositor_pipeline::pipeline;
use compositor_render::scene;

use super::video::*;
use super::*;

impl From<VideoCodec> for pipeline::VideoCodec {
    fn from(value: VideoCodec) -> Self {
        match value {
            VideoCodec::H264 => pipeline::VideoCodec::H264,
        }
    }
}

impl TryFrom<Video> for scene::Component {
    type Error = TypeError;

    fn try_from(value: Video) -> Result<Self, Self::Error> {
        value.root.try_into()
    }
}
