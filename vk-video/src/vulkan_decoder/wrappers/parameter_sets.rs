use ash::vk;
use h264_reader::nal::sps::SeqParameterSet;

use crate::VulkanDecoderError;

const MACROBLOCK_SIZE: u32 = 16;

pub(crate) trait SeqParameterSetExt {
    fn width(&self) -> Result<u32, VulkanDecoderError>;
    fn height(&self) -> Result<u32, VulkanDecoderError>;
}

impl SeqParameterSetExt for SeqParameterSet {
    fn width(&self) -> Result<u32, VulkanDecoderError> {
        match self.frame_cropping {
            None => Ok((self.pic_width_in_mbs_minus1 + 1) * MACROBLOCK_SIZE),
            Some(_) => Err(VulkanDecoderError::FrameCroppingNotSupported),
        }
    }

    fn height(&self) -> Result<u32, VulkanDecoderError> {
        match self.frame_mbs_flags {
            h264_reader::nal::sps::FrameMbsFlags::Frames => {
                Ok((self.pic_height_in_map_units_minus1 + 1) * MACROBLOCK_SIZE)
            }
            h264_reader::nal::sps::FrameMbsFlags::Fields { .. } => {
                Err(VulkanDecoderError::FieldsNotSupported)
            }
        }
    }
}

pub(crate) struct VkSequenceParameterSet {
    pub(crate) sps: vk::native::StdVideoH264SequenceParameterSet,
    // in the future, heap-allocated VUI and HRD parameters can be put here to have everything
    // together
}

impl TryFrom<&'_ SeqParameterSet> for VkSequenceParameterSet {
    type Error = VulkanDecoderError;

    #[allow(non_snake_case)]
    fn try_from(sps: &SeqParameterSet) -> Result<VkSequenceParameterSet, VulkanDecoderError> {
        let flags = vk::native::StdVideoH264SpsFlags {
            _bitfield_1: vk::native::StdVideoH264SpsFlags::new_bitfield_1(
                sps.constraint_flags.flag0().into(),
                sps.constraint_flags.flag1().into(),
                sps.constraint_flags.flag2().into(),
                sps.constraint_flags.flag3().into(),
                sps.constraint_flags.flag4().into(),
                sps.constraint_flags.flag5().into(),
                sps.direct_8x8_inference_flag.into(),
                match sps.frame_mbs_flags {
                    h264_reader::nal::sps::FrameMbsFlags::Frames => 0,
                    h264_reader::nal::sps::FrameMbsFlags::Fields {
                        mb_adaptive_frame_field_flag,
                    } => mb_adaptive_frame_field_flag.into(),
                },
                matches!(
                    sps.frame_mbs_flags,
                    h264_reader::nal::sps::FrameMbsFlags::Frames
                )
                .into(),
                match sps.pic_order_cnt {
                    h264_reader::nal::sps::PicOrderCntType::TypeOne {
                        delta_pic_order_always_zero_flag,
                        ..
                    } => delta_pic_order_always_zero_flag.into(),
                    // The spec doesn't say what to do if this flag is not present...
                    h264_reader::nal::sps::PicOrderCntType::TypeZero { .. }
                    | h264_reader::nal::sps::PicOrderCntType::TypeTwo => 0,
                },
                sps.chroma_info.separate_colour_plane_flag.into(),
                sps.gaps_in_frame_num_value_allowed_flag.into(),
                sps.chroma_info.qpprime_y_zero_transform_bypass_flag.into(),
                sps.frame_cropping.is_some().into(),
                sps.chroma_info.scaling_matrix.is_some().into(),
                0,
            ),
            _bitfield_align_1: [],
            __bindgen_padding_0: 0,
        };

        let profile_idc: u8 = sps.profile_idc.into();

        let pic_order_cnt_type = match sps.pic_order_cnt {
            h264_reader::nal::sps::PicOrderCntType::TypeZero { .. } => 0,
            h264_reader::nal::sps::PicOrderCntType::TypeOne { .. } => 1,
            h264_reader::nal::sps::PicOrderCntType::TypeTwo => 2,
        };

        let (
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
        ) = match &sps.pic_order_cnt {
            h264_reader::nal::sps::PicOrderCntType::TypeOne {
                offset_for_non_ref_pic,
                offset_for_top_to_bottom_field,
                offsets_for_ref_frame,
                ..
            } => (
                *offset_for_non_ref_pic,
                *offset_for_top_to_bottom_field,
                offsets_for_ref_frame.len() as u8,
            ),
            h264_reader::nal::sps::PicOrderCntType::TypeZero { .. } => (0, 0, 0),
            h264_reader::nal::sps::PicOrderCntType::TypeTwo => (0, 0, 0),
        };

        let log2_max_pic_order_cnt_lsb_minus4 = match &sps.pic_order_cnt {
            h264_reader::nal::sps::PicOrderCntType::TypeZero {
                log2_max_pic_order_cnt_lsb_minus4,
            } => *log2_max_pic_order_cnt_lsb_minus4,
            h264_reader::nal::sps::PicOrderCntType::TypeOne { .. }
            | h264_reader::nal::sps::PicOrderCntType::TypeTwo => 0,
        };

        let (
            frame_crop_left_offset,
            frame_crop_right_offset,
            frame_crop_top_offset,
            frame_crop_bottom_offset,
        ) = match sps.frame_cropping {
            Some(h264_reader::nal::sps::FrameCropping {
                left_offset,
                right_offset,
                top_offset,
                bottom_offset,
            }) => (left_offset, right_offset, top_offset, bottom_offset),
            None => (0, 0, 0, 0),
        };

        let pOffsetForRefFrame = match &sps.pic_order_cnt {
            h264_reader::nal::sps::PicOrderCntType::TypeOne {
                offsets_for_ref_frame,
                ..
            } => offsets_for_ref_frame.as_ptr(),
            h264_reader::nal::sps::PicOrderCntType::TypeZero { .. }
            | h264_reader::nal::sps::PicOrderCntType::TypeTwo => std::ptr::null(),
        };

        let pScalingLists = match sps.chroma_info.scaling_matrix {
            Some(_) => return Err(VulkanDecoderError::ScalingListsNotSupported),
            None => std::ptr::null(),
        };

        // TODO: this is not necessary to reconstruct samples. I don't know why the decoder would
        // need this. Maybe we can do this in the future.
        let pSequenceParameterSetVui = std::ptr::null();

        Ok(Self {
            sps: vk::native::StdVideoH264SequenceParameterSet {
                flags,
                profile_idc: profile_idc as u32,
                level_idc: h264_level_idc_to_vk(sps.level_idc),
                chroma_format_idc: sps.chroma_info.chroma_format.to_chroma_format_idc(),
                seq_parameter_set_id: sps.seq_parameter_set_id.id(),
                bit_depth_luma_minus8: sps.chroma_info.bit_depth_luma_minus8,
                bit_depth_chroma_minus8: sps.chroma_info.bit_depth_chroma_minus8,
                log2_max_frame_num_minus4: sps.log2_max_frame_num_minus4,
                pic_order_cnt_type,
                offset_for_non_ref_pic,
                offset_for_top_to_bottom_field,
                num_ref_frames_in_pic_order_cnt_cycle,
                log2_max_pic_order_cnt_lsb_minus4,
                max_num_ref_frames: sps.max_num_ref_frames as u8,
                reserved1: 0,
                pic_width_in_mbs_minus1: sps.pic_width_in_mbs_minus1,
                pic_height_in_map_units_minus1: sps.pic_height_in_map_units_minus1,
                frame_crop_left_offset,
                frame_crop_right_offset,
                frame_crop_top_offset,
                frame_crop_bottom_offset,
                reserved2: 0,
                pOffsetForRefFrame,
                pScalingLists,
                pSequenceParameterSetVui,
            },
        })
    }
}

trait ChromaFormatExt {
    fn to_chroma_format_idc(&self) -> u32;
}

impl ChromaFormatExt for h264_reader::nal::sps::ChromaFormat {
    fn to_chroma_format_idc(&self) -> u32 {
        match self {
            h264_reader::nal::sps::ChromaFormat::Monochrome => 0,
            h264_reader::nal::sps::ChromaFormat::YUV420 => 1,
            h264_reader::nal::sps::ChromaFormat::YUV422 => 2,
            h264_reader::nal::sps::ChromaFormat::YUV444 => 3,
            h264_reader::nal::sps::ChromaFormat::Invalid(v) => *v,
        }
    }
}

fn h264_level_idc_to_vk(level_idc: u8) -> u32 {
    match level_idc {
        10 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_1_0,
        11 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_1_1,
        12 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_1_2,
        13 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_1_3,
        20 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_2_0,
        21 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_2_1,
        22 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_2_2,
        30 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_3_0,
        31 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_3_1,
        32 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_3_2,
        40 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_4_0,
        41 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_4_1,
        42 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_4_2,
        50 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_5_0,
        51 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_5_1,
        52 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_5_2,
        60 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_6_0,
        61 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_6_1,
        62 => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_6_2,
        _ => vk::native::StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_INVALID,
    }
}

pub(crate) struct VkPictureParameterSet {
    pub(crate) pps: vk::native::StdVideoH264PictureParameterSet,
}

impl TryFrom<&'_ h264_reader::nal::pps::PicParameterSet> for VkPictureParameterSet {
    type Error = VulkanDecoderError;

    #[allow(non_snake_case)]
    fn try_from(pps: &h264_reader::nal::pps::PicParameterSet) -> Result<Self, Self::Error> {
        let flags = vk::native::StdVideoH264PpsFlags {
            _bitfield_align_1: [],
            __bindgen_padding_0: [0; 3],
            _bitfield_1: vk::native::StdVideoH264PpsFlags::new_bitfield_1(
                pps.extension
                    .as_ref()
                    .map(|ext| ext.transform_8x8_mode_flag.into())
                    .unwrap_or(0),
                pps.redundant_pic_cnt_present_flag.into(),
                pps.constrained_intra_pred_flag.into(),
                pps.deblocking_filter_control_present_flag.into(),
                pps.weighted_pred_flag.into(),
                pps.bottom_field_pic_order_in_frame_present_flag.into(),
                pps.entropy_coding_mode_flag.into(),
                pps.extension
                    .as_ref()
                    .map(|ext| ext.pic_scaling_matrix.is_some().into())
                    .unwrap_or(0),
            ),
        };

        let chroma_qp_index_offset = pps.chroma_qp_index_offset as i8;

        let second_chroma_qp_index_offset = pps
            .extension
            .as_ref()
            .map(|ext| ext.second_chroma_qp_index_offset as i8)
            .unwrap_or(chroma_qp_index_offset);

        let pScalingLists = match pps.extension {
            Some(h264_reader::nal::pps::PicParameterSetExtra {
                pic_scaling_matrix: Some(_),
                ..
            }) => return Err(VulkanDecoderError::ScalingListsNotSupported),
            _ => std::ptr::null(),
        };

        Ok(Self {
            pps: vk::native::StdVideoH264PictureParameterSet {
                flags,
                seq_parameter_set_id: pps.seq_parameter_set_id.id(),
                pic_parameter_set_id: pps.pic_parameter_set_id.id(),
                num_ref_idx_l0_default_active_minus1: pps.num_ref_idx_l0_default_active_minus1
                    as u8,
                num_ref_idx_l1_default_active_minus1: pps.num_ref_idx_l1_default_active_minus1
                    as u8,
                weighted_bipred_idc: pps.weighted_bipred_idc.into(),
                pic_init_qp_minus26: pps.pic_init_qp_minus26 as i8,
                pic_init_qs_minus26: pps.pic_init_qs_minus26 as i8,
                chroma_qp_index_offset,
                second_chroma_qp_index_offset,
                pScalingLists,
            },
        })
    }
}
