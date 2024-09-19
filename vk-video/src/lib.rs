mod parser;
mod vulkan_decoder;

use parser::Parser;
use vulkan_decoder::VulkanDecoder;

pub use parser::ParserError;
pub use vulkan_decoder::{VulkanCtx, VulkanCtxError, VulkanDecoderError};

pub use vulkan_decoder::WgpuCtx;

pub struct Decoder<'a> {
    vulkan_decoder: VulkanDecoder<'a>,
    parser: Parser,
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    #[error("Error originating in the decoder: {0}")]
    VulkanDecoderError(#[from] VulkanDecoderError),

    #[error("Error originating in the h264 parser: {0}")]
    ParserError(#[from] ParserError),
}

impl<'a> Decoder<'a> {
    pub fn new(vulkan_ctx: std::sync::Arc<VulkanCtx>) -> Result<Self, DecoderError> {
        let parser = Parser::default();
        let vulkan_decoder = VulkanDecoder::new(vulkan_ctx)?;

        Ok(Self {
            parser,
            vulkan_decoder,
        })
    }
}

impl Decoder<'_> {
    /// The result is a [`Vec`] of [`Vec<u8>`]. Each [`Vec<u8>`] contains a single frame in the
    /// NV12 format.
    pub fn decode_to_bytes(
        &mut self,
        h264_bytestream: &[u8],
    ) -> Result<Vec<Vec<u8>>, DecoderError> {
        let instructions = self
            .parser
            .parse(h264_bytestream)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(self.vulkan_decoder.decode_to_bytes(&instructions)?)
    }

    // TODO: the below hasn't been verified.
    /// The produced textures have the [`wgpu::TextureFormat::NV12`] format and can be used as a copy source or a texture binding.
    pub fn decode_to_wgpu_textures(
        &mut self,
        h264_bytestream: &[u8],
    ) -> Result<Vec<wgpu::Texture>, DecoderError> {
        let instructions = self
            .parser
            .parse(h264_bytestream)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(self.vulkan_decoder.decode_to_wgpu_textures(&instructions)?)
    }
}
