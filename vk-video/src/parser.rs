use std::{
    io::Read,
    sync::{mpsc, Arc},
};

use h264_reader::{
    annexb::AnnexBReader,
    nal::{
        pps::PicParameterSet,
        slice::{DecRefPicMarking, NumRefIdxActive, RefPicListModifications, SliceHeader},
        sps::SeqParameterSet,
        Nal, RefNal,
    },
    push::{AccumulatedNalHandler, NalAccumulator, NalInterest},
};
use tracing::trace;

mod au_splitter;

#[derive(Debug, thiserror::Error)]
pub enum ReferenceManagementError {
    #[error("B frames are not supported")]
    BFramesNotSupported,

    #[error("Long-term references are not supported")]
    LongTermRefsNotSupported,

    #[error("SI frames are not supported")]
    SIFramesNotSupported,

    #[error("SP frames are not supported")]
    SPFramesNotSupported,

    #[error("Adaptive memory control decoded reference picture marking process is not supported")]
    AdaptiveMemCtlNotSupported,

    #[error("Reference picture list modifications are not supported")]
    RefPicListModificationsNotSupported,

    #[error("PicOrderCntType {0} is not supperted")]
    PicOrderCntTypeNotSupported(u8),

    #[error("pic_order_cnt_lsb is not present in a slice header, but is required for decoding")]
    PicOrderCntLsbNotPresent,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceId(usize);

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

#[derive(Debug, Default)]
struct ReferenceContext {
    pictures: ReferencePictures,
    next_reference_id: ReferenceId,
    _previous_frame_num: usize,
    prev_pic_order_cnt_msb: i32,
    prev_pic_order_cnt_lsb: i32,
}

impl ReferenceContext {
    fn get_next_reference_id(&mut self) -> ReferenceId {
        let result = self.next_reference_id;
        self.next_reference_id = ReferenceId(result.0 + 1);
        result
    }

    fn reset_state(&mut self) {
        *self = Self {
            pictures: ReferencePictures::default(),
            next_reference_id: ReferenceId::default(),
            _previous_frame_num: 0,
            prev_pic_order_cnt_msb: 0,
            prev_pic_order_cnt_lsb: 0,
        };
    }

    fn add_short_term_reference(
        &mut self,
        header: Arc<SliceHeader>,
        pic_order_cnt: [i32; 2],
    ) -> ReferenceId {
        let id = self.get_next_reference_id();
        self.pictures.short_term.push(ShortTermReferencePicture {
            header,
            id,
            pic_order_cnt,
        });
        id
    }

    fn put_picture(
        &mut self,
        mut slices: Vec<Slice>,
        sps: &SeqParameterSet,
        pps: &PicParameterSet,
    ) -> Result<Vec<DecoderInstruction>, ParserError> {
        let header = slices.last().unwrap().header.clone();
        let mut rbsp_bytes = Vec::new();
        let mut slice_indices = Vec::new();
        for slice in &mut slices {
            if slice.rbsp_bytes.is_empty() {
                continue;
            }
            slice_indices.push(rbsp_bytes.len());
            rbsp_bytes.append(&mut slice.rbsp_bytes);
        }

        match header.dec_ref_pic_marking {
            Some(DecRefPicMarking::Idr {
                long_term_reference_flag,
                ..
            }) => {
                if long_term_reference_flag {
                    Err(ReferenceManagementError::LongTermRefsNotSupported)?;
                }

                let decode_info = self.decode_information_for_frame(
                    header.clone(),
                    slice_indices,
                    rbsp_bytes,
                    sps,
                    pps,
                )?;

                self.reset_state();

                let reference_id =
                    self.add_short_term_reference(header, decode_info.picture_info.PicOrderCnt);

                Ok(vec![DecoderInstruction::Idr {
                    decode_info,
                    reference_id,
                }])
            }

            Some(DecRefPicMarking::SlidingWindow) => {
                let num_short_term = self.pictures.short_term.len();
                let num_long_term = self.pictures.long_term.len();

                let decode_info = self.decode_information_for_frame(
                    header.clone(),
                    slice_indices,
                    rbsp_bytes,
                    sps,
                    pps,
                )?;
                let reference_id = self
                    .add_short_term_reference(header.clone(), decode_info.picture_info.PicOrderCnt);

                let mut decoder_instructions = vec![DecoderInstruction::DecodeAndStoreAs {
                    decode_info,
                    reference_id,
                }];

                if num_short_term + num_long_term == sps.max_num_ref_frames.max(1) as usize
                    && !self.pictures.short_term.is_empty()
                {
                    let (idx, _) = self
                        .pictures
                        .short_term
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, reference)| {
                            reference
                                .decode_picture_numbers(header.frame_num as i64, sps)
                                .unwrap()
                                .FrameNumWrap
                        })
                        .unwrap();

                    decoder_instructions.push(DecoderInstruction::Drop {
                        reference_ids: vec![self.pictures.short_term.remove(idx).id],
                    })
                }

                Ok(decoder_instructions)
            }

            Some(DecRefPicMarking::Adaptive(_)) => {
                Err(ReferenceManagementError::AdaptiveMemCtlNotSupported)?
            }

            // this picture is not a reference
            None => Ok(vec![DecoderInstruction::Decode {
                decode_info: self.decode_information_for_frame(
                    header,
                    slice_indices,
                    rbsp_bytes,
                    sps,
                    pps,
                )?,
            }]),
        }
    }

    fn decode_information_for_frame(
        &mut self,
        header: Arc<SliceHeader>,
        slice_indices: Vec<usize>,
        rbsp_bytes: Vec<u8>,
        sps: &SeqParameterSet,
        pps: &PicParameterSet,
    ) -> Result<DecodeInformation, ParserError> {
        let reference_list = match header.slice_type.family {
            h264_reader::nal::slice::SliceFamily::P => {
                let reference_list =
                    self.initialize_reference_picture_list_for_frame(&header, sps, pps)?;

                match &header.ref_pic_list_modification {
                    Some(RefPicListModifications::P {
                        ref_pic_list_modification_l0,
                    }) => {
                        if !ref_pic_list_modification_l0.is_empty() {
                            Err(ReferenceManagementError::RefPicListModificationsNotSupported)?;
                        }
                    }

                    None
                    | Some(RefPicListModifications::I)
                    | Some(RefPicListModifications::B { .. }) => unreachable!(),
                }

                Some(reference_list)
            }
            h264_reader::nal::slice::SliceFamily::I => None,
            h264_reader::nal::slice::SliceFamily::B => {
                return Err(ReferenceManagementError::BFramesNotSupported)?
            }
            h264_reader::nal::slice::SliceFamily::SP => {
                return Err(ReferenceManagementError::SPFramesNotSupported)?
            }
            h264_reader::nal::slice::SliceFamily::SI => {
                return Err(ReferenceManagementError::SIFramesNotSupported)?
            }
        };

        let pic_order_cnt = match sps.pic_order_cnt {
            h264_reader::nal::sps::PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4,
            } => {
                // this section is very hard to read, but all of this code is just copied from the
                // h.264 spec, where it looks almost exactly like this

                let max_pic_order_cnt_lsb = 2_i32.pow(log2_max_pic_order_cnt_lsb_minus4 as u32 + 4);

                let (prev_pic_order_cnt_msb, prev_pic_order_cnt_lsb) =
                    if header.idr_pic_id.is_some() {
                        (0, 0)
                    } else {
                        (self.prev_pic_order_cnt_msb, self.prev_pic_order_cnt_lsb)
                    };

                let (pic_order_cnt_lsb, delta_pic_order_cnt_bottom) = match header
                    .pic_order_cnt_lsb
                    .as_ref()
                    .ok_or(ReferenceManagementError::PicOrderCntLsbNotPresent)?
                {
                    h264_reader::nal::slice::PicOrderCountLsb::Frame(pic_order_cnt_lsb) => {
                        (*pic_order_cnt_lsb, 0)
                    }
                    h264_reader::nal::slice::PicOrderCountLsb::FieldsAbsolute {
                        pic_order_cnt_lsb,
                        delta_pic_order_cnt_bottom,
                    } => (*pic_order_cnt_lsb, *delta_pic_order_cnt_bottom),
                    h264_reader::nal::slice::PicOrderCountLsb::FieldsDelta(_) => {
                        Err(ReferenceManagementError::PicOrderCntLsbNotPresent)?
                    }
                };

                let pic_order_cnt_lsb = pic_order_cnt_lsb as i32;

                let pic_order_cnt_msb = if pic_order_cnt_lsb < prev_pic_order_cnt_lsb
                    && prev_pic_order_cnt_lsb - pic_order_cnt_lsb >= max_pic_order_cnt_lsb / 2
                {
                    prev_pic_order_cnt_msb + max_pic_order_cnt_lsb
                } else if pic_order_cnt_lsb > prev_pic_order_cnt_lsb
                    && pic_order_cnt_lsb - prev_pic_order_cnt_lsb > max_pic_order_cnt_lsb / 2
                {
                    prev_pic_order_cnt_msb - max_pic_order_cnt_lsb
                } else {
                    prev_pic_order_cnt_msb
                };

                let pic_order_cnt = if header.field_pic == h264_reader::nal::slice::FieldPic::Frame
                {
                    let top_field_order_cnt = pic_order_cnt_msb + pic_order_cnt_lsb;

                    let bottom_field_order_cnt = top_field_order_cnt + delta_pic_order_cnt_bottom;

                    top_field_order_cnt.min(bottom_field_order_cnt)
                } else {
                    pic_order_cnt_msb + pic_order_cnt_lsb
                };

                self.prev_pic_order_cnt_msb = pic_order_cnt_msb;
                self.prev_pic_order_cnt_lsb = pic_order_cnt_lsb;

                pic_order_cnt
            }

            h264_reader::nal::sps::PicOrderCntType::TypeOne { .. } => {
                Err(ReferenceManagementError::PicOrderCntTypeNotSupported(1))?
            }

            h264_reader::nal::sps::PicOrderCntType::TypeTwo => match header.dec_ref_pic_marking {
                None => 2 * header.frame_num as i32 - 1,
                Some(DecRefPicMarking::Idr { .. }) | Some(DecRefPicMarking::SlidingWindow) => {
                    2 * header.frame_num as i32
                }
                Some(DecRefPicMarking::Adaptive(..)) => {
                    Err(ReferenceManagementError::AdaptiveMemCtlNotSupported)?
                }
            },
        };

        let pic_order_cnt = [pic_order_cnt; 2];

        Ok(DecodeInformation {
            reference_list,
            header: header.clone(),
            slice_indices,
            rbsp_bytes,
            sps_id: sps.id().id(),
            pps_id: pps.pic_parameter_set_id.id(),
            picture_info: PictureInfo {
                non_existing: false,
                used_for_long_term_reference: false,
                PicOrderCnt: pic_order_cnt,
                FrameNum: header.frame_num,
            },
        })
    }

    fn initialize_short_term_reference_picture_list_for_frame(
        &self,
        header: &SliceHeader,
        sps: &SeqParameterSet,
    ) -> Result<Vec<ReferencePictureInfo>, ParserError> {
        let mut short_term_reference_list = self
            .pictures
            .short_term
            .iter()
            .map(|reference| {
                Ok((
                    reference,
                    reference.decode_picture_numbers(header.frame_num.into(), sps)?,
                ))
            })
            .collect::<Result<Vec<_>, ParserError>>()?;

        short_term_reference_list.sort_by_key(|(_, numbers)| -numbers.PicNum);

        let short_term_reference_list = short_term_reference_list
            .into_iter()
            .map(|(reference, numbers)| ReferencePictureInfo {
                id: reference.id,
                picture_info: PictureInfo {
                    FrameNum: numbers.FrameNum as u16,
                    used_for_long_term_reference: false,
                    non_existing: false,
                    PicOrderCnt: reference.pic_order_cnt,
                },
            })
            .collect::<Vec<_>>();

        Ok(short_term_reference_list)
    }

    fn initialize_long_term_reference_picture_list_for_frame(
        &self,
    ) -> Result<Vec<ReferencePictureInfo>, ReferenceManagementError> {
        if !self.pictures.long_term.is_empty() {
            panic!("long-term references are not supported!");
        }

        Ok(Vec::new())
    }

    fn initialize_reference_picture_list_for_frame(
        &self,
        header: &SliceHeader,
        sps: &SeqParameterSet,
        pps: &PicParameterSet,
    ) -> Result<Vec<ReferencePictureInfo>, ParserError> {
        let num_ref_idx_l0_active = header
            .num_ref_idx_active
            .as_ref()
            .map(|num| match num {
                NumRefIdxActive::P {
                    num_ref_idx_l0_active_minus1,
                } => Ok(*num_ref_idx_l0_active_minus1),
                NumRefIdxActive::B { .. } => Err(ReferenceManagementError::BFramesNotSupported),
            })
            .unwrap_or(Ok(pps.num_ref_idx_l0_default_active_minus1))?
            + 1;

        let short_term_reference_list =
            self.initialize_short_term_reference_picture_list_for_frame(header, sps)?;

        let long_term_reference_list =
            self.initialize_long_term_reference_picture_list_for_frame()?;

        let mut reference_list = short_term_reference_list
            .into_iter()
            .chain(long_term_reference_list)
            .collect::<Vec<_>>();

        reference_list.truncate(num_ref_idx_l0_active as usize);

        Ok(reference_list)
    }
}

#[derive(Debug)]
struct ShortTermReferencePicture {
    header: Arc<SliceHeader>,
    id: ReferenceId,
    pic_order_cnt: [i32; 2],
}

impl ShortTermReferencePicture {
    #[allow(non_snake_case)]
    fn decode_picture_numbers(
        &self,
        current_frame_num: i64,
        sps: &SeqParameterSet,
    ) -> Result<ShortTermReferencePictureNumbers, ParserError> {
        if self.header.field_pic != h264_reader::nal::slice::FieldPic::Frame {
            return Err(ParserError::FieldsNotSupported);
        }

        let MaxFrameNum = sps.max_frame_num();

        let FrameNum = self.header.frame_num as i64;

        let FrameNumWrap = if FrameNum > current_frame_num {
            FrameNum - MaxFrameNum
        } else {
            FrameNum
        };

        // this assumes we're dealing with a short-term reference frame
        let PicNum = FrameNumWrap;

        Ok(ShortTermReferencePictureNumbers {
            FrameNum,
            FrameNumWrap,
            PicNum,
        })
    }
}

#[derive(Debug)]
struct LongTermReferencePicture {
    _header: Arc<SliceHeader>,
    _id: ReferenceId,
}

#[allow(non_snake_case)]
struct ShortTermReferencePictureNumbers {
    FrameNum: i64,

    FrameNumWrap: i64,

    PicNum: i64,
}

#[derive(Debug, Default)]
struct ReferencePictures {
    long_term: Vec<LongTermReferencePicture>,
    short_term: Vec<ShortTermReferencePicture>,
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
