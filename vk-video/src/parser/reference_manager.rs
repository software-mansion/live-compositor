use std::sync::Arc;

use h264_reader::nal::{
    pps::PicParameterSet,
    slice::{DecRefPicMarking, NumRefIdxActive, RefPicListModifications, SliceHeader},
    sps::SeqParameterSet,
};

use super::{
    DecodeInformation, DecoderInstruction, ParserError, PictureInfo, ReferencePictureInfo, Slice,
    SpsExt,
};

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

#[derive(Debug, Default)]
pub(crate) struct ReferenceContext {
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

    pub(crate) fn put_picture(
        &mut self,
        mut slices: Vec<Slice>,
        sps: &SeqParameterSet,
        pps: &PicParameterSet,
    ) -> Result<Vec<DecoderInstruction>, ParserError> {
        let header = slices.last().unwrap().header.clone();

        // maybe this should be done in a different place, but if you think about it, there's not
        // really that many places to put this code in
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
