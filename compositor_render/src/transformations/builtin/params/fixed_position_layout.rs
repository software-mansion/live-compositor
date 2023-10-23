use compositor_common::scene::{
    builtin_transformations::{
        FixedPositionLayoutSpec, HorizontalPosition, TextureLayout, VerticalPosition,
    },
    Resolution,
};

use crate::transformations::builtin::box_layout::BoxLayout;

use super::box_layout_params::BoxLayoutParams;

pub fn new_fixed_position_layout_params(
    spec: &FixedPositionLayoutSpec,
    input_resolutions: &[Option<Resolution>],
) -> BoxLayoutParams {
    let boxes = spec
        .texture_layouts
        .iter()
        .zip(input_resolutions.iter())
        .map(|(texture_layout, &input_resolution)| {
            new_box_layout(texture_layout, input_resolution.as_ref(), spec.resolution)
        })
        .collect();
    BoxLayoutParams {
        boxes,
        output_resolution: spec.resolution,
    }
}

fn new_box_layout(
    layout: &TextureLayout,
    input_resolution: Option<&Resolution>,
    output_resolution: Resolution,
) -> BoxLayout {
    let Some(input_resolution) = input_resolution else {
        return BoxLayout::NONE;
    };

    let (top, left) = spec_to_top_left_coords(layout, input_resolution, &output_resolution);

    BoxLayout {
        top_left_corner: (left, top),
        width: input_resolution.width as f32 * layout.scale,
        height: input_resolution.height as f32 * layout.scale,
        rotation_degrees: layout.rotation.0 as f32,
    }
}

fn spec_to_top_left_coords(
    layout: &TextureLayout,
    input_resolution: &Resolution,
    output_resolution: &Resolution,
) -> (f32, f32) {
    let top = match layout.vertical_position {
        VerticalPosition::Top(top) => top.pixels(output_resolution.height as u32),
        VerticalPosition::Bottom(bottom) => {
            output_resolution.height as i32
                - (input_resolution.height as f32 * layout.scale) as i32
                - bottom.pixels(output_resolution.height as u32)
        }
    };
    let left = match layout.horizontal_position {
        HorizontalPosition::Left(left) => left.pixels(output_resolution.width as u32),
        HorizontalPosition::Right(right) => {
            output_resolution.width as i32
                - (input_resolution.width as f32 * layout.scale) as i32
                - right.pixels(output_resolution.width as u32)
        }
    };

    (top as f32, left as f32)
}
