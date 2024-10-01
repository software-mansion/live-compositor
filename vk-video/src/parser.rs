use std::{
    io::Read,
    sync::{mpsc, Arc},
};

use h264_reader::{
    annexb::AnnexBReader,
    nal::{pps::PicParameterSet, slice::SliceHeader, sps::SeqParameterSet, Nal, RefNal},
    push::{AccumulatedNalHandler, NalAccumulator, NalInterest},
};
use reference_manager::ReferenceContext;
use tracing::trace;

mod au_splitter;
mod reference_manager;

pub use reference_manager::{ReferenceId, ReferenceManagementError};

#[derive(Clone, derivative::Derivative)]
#[derivative(Debug)]
#[allow(non_snake_case)]
pub struct DecodeInformation {
    pub(crate) reference_list: Option<Vec<ReferencePictureInfo>>,
    #[derivative(Debug = "ignore")]
    pub(crate) rbsp_bytes: Vec<u8>,
    pub(crate) slice_indices: Vec<usize>,
    #[derivative(Debug = "ignore")]
    pub(crate) header: Arc<SliceHeader>,
    pub(crate) sps_id: u8,
    pub(crate) pps_id: u8,
    pub(crate) picture_info: PictureInfo,
}

#[derive(Debug, Clone)]
pub(crate) struct ReferencePictureInfo {
    pub(crate) id: ReferenceId,
    pub(crate) picture_info: PictureInfo,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub(crate) struct PictureInfo {
    pub(crate) used_for_long_term_reference: bool,
    pub(crate) non_existing: bool,
    pub(crate) FrameNum: u16,
    pub(crate) PicOrderCnt: [i32; 2],
}

#[derive(Debug, Clone)]
pub enum DecoderInstruction {
    Decode {
        decode_info: DecodeInformation,
    },

    DecodeAndStoreAs {
        decode_info: DecodeInformation,
        reference_id: ReferenceId,
    },

    Idr {
        decode_info: DecodeInformation,
        reference_id: ReferenceId,
    },

    Drop {
        reference_ids: Vec<ReferenceId>,
    },

    Sps(SeqParameterSet),

    Pps(PicParameterSet),
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error(transparent)]
    ReferenceManagementError(#[from] ReferenceManagementError),

    #[error("Bitstreams that allow gaps in frame_num are not supported")]
    GapsInFrameNumNotSupported,

    #[error("Streams containing fields instead of frames are not supported")]
    FieldsNotSupported,

    #[error("Error while parsing a NAL header: {0:?}")]
    NalHeaderParseError(h264_reader::nal::NalHeaderError),

    #[error("Error while parsing SPS: {0:?}")]
    SpsParseError(h264_reader::nal::sps::SpsError),

    #[error("Error while parsing PPS: {0:?}")]
    PpsParseError(h264_reader::nal::pps::PpsError),

    #[error("Error while parsing a slice: {0:?}")]
    SliceParseError(h264_reader::nal::slice::SliceHeaderError),
}

struct NalReceiver {
    parser_ctx: h264_reader::Context,
    au_splitter: au_splitter::AUSplitter,
    reference_ctx: ReferenceContext,
    debug_channel: mpsc::Sender<NaluDebugInfo>,
    decoder_channel: mpsc::Sender<Result<DecoderInstruction, ParserError>>,
}

impl AccumulatedNalHandler for NalReceiver {
    fn nal(&mut self, nal: RefNal<'_>) -> NalInterest {
        if !nal.is_complete() {
            return NalInterest::Buffer;
        }

        match self.handle_nal(nal) {
            Ok((debug_nalu, instructions)) => {
                self.debug_channel.send(debug_nalu).unwrap();
                for instruction in instructions {
                    self.decoder_channel.send(Ok(instruction)).unwrap();
                }
            }

            Err(err) => {
                self.decoder_channel.send(Err(err)).unwrap();
            }
        }

        NalInterest::Ignore
    }
}

impl NalReceiver {
    fn handle_nal(
        &mut self,
        nal: RefNal<'_>,
    ) -> Result<(NaluDebugInfo, Vec<DecoderInstruction>), ParserError> {
        let nal_unit_type = nal
            .header()
            .map_err(ParserError::NalHeaderParseError)?
            .nal_unit_type();

        match nal_unit_type {
            h264_reader::nal::UnitType::SeqParameterSet => {
                let parsed = h264_reader::nal::sps::SeqParameterSet::from_bits(nal.rbsp_bits())
                    .map_err(ParserError::SpsParseError)?;

                // Perhaps this shouldn't be here, but this is the only place we process sps
                // before sending them to the decoder. It also seems that this is the only thing we
                // need to check about the sps.
                if parsed.gaps_in_frame_num_value_allowed_flag {
                    // TODO: what else to do here? sure we'll throw an error, but shouldn't we also
                    // terminate the parser somehow?
                    // perhaps this should be considered in other places we throw errors too
                    Err(ParserError::GapsInFrameNumNotSupported)
                } else {
                    self.parser_ctx.put_seq_param_set(parsed.clone());
                    Ok((
                        NaluDebugInfo::Sps(parsed.clone()),
                        vec![DecoderInstruction::Sps(parsed)],
                    ))
                }
            }

            h264_reader::nal::UnitType::PicParameterSet => {
                let parsed = h264_reader::nal::pps::PicParameterSet::from_bits(
                    &self.parser_ctx,
                    nal.rbsp_bits(),
                )
                .map_err(ParserError::PpsParseError)?;

                self.parser_ctx.put_pic_param_set(parsed.clone());

                Ok((
                    NaluDebugInfo::Pps(parsed.clone()),
                    vec![DecoderInstruction::Pps(parsed)],
                ))
            }

            h264_reader::nal::UnitType::SliceLayerWithoutPartitioningNonIdr
            | h264_reader::nal::UnitType::SliceLayerWithoutPartitioningIdr => {
                let (header, sps, pps) = h264_reader::nal::slice::SliceHeader::from_bits(
                    &self.parser_ctx,
                    &mut nal.rbsp_bits(),
                    nal.header().unwrap(),
                )
                .map_err(ParserError::SliceParseError)?;

                let header = Arc::new(header);

                let debug_nalu = match nal_unit_type {
                    h264_reader::nal::UnitType::SliceLayerWithoutPartitioningIdr => {
                        NaluDebugInfo::SliceWithoutPartitioningHeaderIdr(header.clone())
                    }
                    h264_reader::nal::UnitType::SliceLayerWithoutPartitioningNonIdr => {
                        NaluDebugInfo::SliceWithoutPartitioningHeaderNonIdr(header.clone())
                    }
                    _ => unreachable!(),
                };

                let mut rbsp_bytes = vec![0, 0, 0, 1];
                nal.reader().read_to_end(&mut rbsp_bytes).unwrap();
                let slice = Slice {
                    nal_header: nal.header().unwrap(),
                    header,
                    pps_id: pps.pic_parameter_set_id,
                    rbsp_bytes,
                };

                let Some(slices) = self.au_splitter.put_slice(slice) else {
                    return Ok((debug_nalu, Vec::new()));
                };

                let instructions = self.reference_ctx.put_picture(slices, sps, pps)?;

                Ok((debug_nalu, instructions))
            }

            h264_reader::nal::UnitType::Unspecified(_)
            | h264_reader::nal::UnitType::SliceDataPartitionALayer
            | h264_reader::nal::UnitType::SliceDataPartitionBLayer
            | h264_reader::nal::UnitType::SliceDataPartitionCLayer
            | h264_reader::nal::UnitType::SEI
            | h264_reader::nal::UnitType::AccessUnitDelimiter
            | h264_reader::nal::UnitType::EndOfSeq
            | h264_reader::nal::UnitType::EndOfStream
            | h264_reader::nal::UnitType::FillerData
            | h264_reader::nal::UnitType::SeqParameterSetExtension
            | h264_reader::nal::UnitType::PrefixNALUnit
            | h264_reader::nal::UnitType::SubsetSeqParameterSet
            | h264_reader::nal::UnitType::DepthParameterSet
            | h264_reader::nal::UnitType::SliceLayerWithoutPartitioningAux
            | h264_reader::nal::UnitType::SliceExtension
            | h264_reader::nal::UnitType::SliceExtensionViewComponent
            | h264_reader::nal::UnitType::Reserved(_) => Ok((
                NaluDebugInfo::Other(format!("{:?}", nal.header().unwrap().nal_unit_type())),
                Vec::new(),
            )),
        }
    }
}

trait SpsExt {
    fn max_frame_num(&self) -> i64;
}

impl SpsExt for SeqParameterSet {
    fn max_frame_num(&self) -> i64 {
        1 << self.log2_max_frame_num()
    }
}

#[derive(Debug)]
// this struct is only ever printed out in debug mode, but clippy detects this as it not being
// used.
#[allow(dead_code)]
pub enum NaluDebugInfo {
    Sps(SeqParameterSet),
    Pps(PicParameterSet),
    SliceWithoutPartitioningHeaderNonIdr(Arc<SliceHeader>),
    SliceWithoutPartitioningHeaderIdr(Arc<SliceHeader>),
    Other(String),
}

pub struct Slice {
    pub nal_header: h264_reader::nal::NalHeader,
    pub pps_id: h264_reader::nal::pps::PicParamSetId,
    pub header: Arc<SliceHeader>,
    pub rbsp_bytes: Vec<u8>,
}

pub struct Parser {
    reader: AnnexBReader<NalAccumulator<NalReceiver>>,
    debug_channel: mpsc::Receiver<NaluDebugInfo>,
    decoder_channel: mpsc::Receiver<Result<DecoderInstruction, ParserError>>,
}

impl Default for Parser {
    fn default() -> Self {
        let (debug_tx, debug_rx) = mpsc::channel();
        let (decoder_tx, decoder_rx) = mpsc::channel();

        Parser {
            reader: AnnexBReader::accumulate(NalReceiver {
                reference_ctx: ReferenceContext::default(),
                au_splitter: au_splitter::AUSplitter::default(),
                debug_channel: debug_tx,
                decoder_channel: decoder_tx,
                parser_ctx: h264_reader::Context::new(),
            }),
            debug_channel: debug_rx,
            decoder_channel: decoder_rx,
        }
    }
}

impl Parser {
    pub fn parse(&mut self, bytes: &[u8]) -> Vec<Result<DecoderInstruction, ParserError>> {
        self.reader.push(bytes);

        let mut instructions = Vec::new();
        while let Ok(instruction) = self.decoder_channel.try_recv() {
            instructions.push(instruction);
        }
        while let Ok(nalu) = self.debug_channel.try_recv() {
            trace!("parsed nalu: {nalu:#?}");
        }

        instructions
    }
}
