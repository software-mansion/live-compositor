use compositor_common::scene::Resolution;

use nalgebra_glm::Mat4;

use crate::transformations::builtin::utils::BoxLayout;

use super::transform_to_resolution::FitParams;

#[derive(Debug)]
struct RowsCols {
    rows: u32,
    cols: u32,
}

impl RowsCols {
    pub fn from_rows_count(inputs_count: u32, rows: u32) -> Self {
        let cols = ceil_div(inputs_count, rows);
        Self { rows, cols }
    }
}

#[derive(Debug)]
pub struct TiledLayoutParams {
    transformation_matrices: Vec<Mat4>,
}

impl TiledLayoutParams {
    pub fn new(
        input_resolutions: &[Option<Resolution>],
        tile_aspect_ratio: (u32, u32),
        output_resolution: Resolution,
    ) -> Self {
        let inputs = input_resolutions
            .iter()
            .filter_map(|input_resolution| *input_resolution);

        let inputs_count = inputs.clone().count() as u32;

        // This should fallback anyway
        if inputs_count == 0 {
            return Self {
                transformation_matrices: vec![Mat4::identity()],
            };
        }

        let optimal_rows_cols =
            Self::optimize_inputs_layout(inputs_count, tile_aspect_ratio, output_resolution);

        let tile_size = Self::tile_size(&optimal_rows_cols, tile_aspect_ratio, output_resolution);

        let tiles_layout = Self::layout_tiles(
            inputs_count,
            &optimal_rows_cols,
            tile_size,
            output_resolution,
        );

        let transformation_matrices: Vec<Mat4> = tiles_layout
            .iter()
            .zip(inputs)
            .map(|(tile_layout, input_resolution)| {
                Self::transformation_matrix(
                    tile_layout,
                    input_resolution,
                    output_resolution,
                    tile_size,
                )
            })
            .collect();

        Self {
            transformation_matrices,
        }
    }

    fn layout_tiles(
        inputs_count: u32,
        rows_cols: &RowsCols,
        tile_size: Resolution,
        output_resolution: Resolution,
    ) -> Vec<BoxLayout> {
        let mut layouts = Vec::with_capacity(inputs_count as usize);

        let tiles_width = tile_size.width as f32 * rows_cols.cols as f32;
        let x_padding = (output_resolution.width as f32 - tiles_width) / 2.0;

        let tiles_height = tile_size.height as f32 * rows_cols.rows as f32;
        let y_padding = (output_resolution.height as f32 - tiles_height) / 2.0;

        for row in 0..(rows_cols.rows - 1) {
            for col in 0..rows_cols.cols {
                let top_left_corner = (
                    x_padding + col as f32 * tile_size.width as f32,
                    y_padding + row as f32 * tile_size.height as f32,
                );

                layouts.push(BoxLayout {
                    top_left_corner,
                    width: tile_size.width as f32,
                    height: tile_size.height as f32,
                    rotation_degrees: 0.0,
                });
            }
        }

        let bottom_row_tiles_count = inputs_count - ((rows_cols.rows - 1) * rows_cols.cols);

        let bottom_tiles_width = tile_size.width as u32 * bottom_row_tiles_count;
        let bottom_row_x_padding = (output_resolution.width as u32 - bottom_tiles_width) / 2;

        for bottom_tile in 0..bottom_row_tiles_count {
            let top_left_corner = (
                tile_size.width as f32 * bottom_tile as f32 + bottom_row_x_padding as f32,
                (rows_cols.rows as f32 - 1.0) * tile_size.height as f32 + y_padding,
            );

            layouts.push(BoxLayout {
                top_left_corner,
                width: tile_size.width as f32,
                height: tile_size.height as f32,
                rotation_degrees: 0.0,
            })
        }

        layouts
    }

    fn tile_size(
        rows_cols: &RowsCols,
        tile_aspect_ratio: (u32, u32),
        output_resolution: Resolution,
    ) -> Resolution {
        let x_scale =
            output_resolution.width as f32 / rows_cols.cols as f32 / tile_aspect_ratio.0 as f32;
        let y_scale =
            output_resolution.height as f32 / rows_cols.rows as f32 / tile_aspect_ratio.1 as f32;

        let scale = if x_scale < y_scale { x_scale } else { y_scale };

        Resolution {
            width: (tile_aspect_ratio.0 as f32 * scale) as usize,
            height: (tile_aspect_ratio.1 as f32 * scale) as usize,
        }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        let mut matrices_bytes = bytes::BytesMut::new();
        for matrix in &self.transformation_matrices {
            let colum_based = matrix.transpose();
            for el in &colum_based {
                matrices_bytes.extend_from_slice(&el.to_ne_bytes())
            }
        }

        matrices_bytes.freeze()
    }

    /// Optimize number of rows and cols to maximize space covered by tiles,
    /// preserving tile aspect_ratio
    fn optimize_inputs_layout(
        inputs_count: u32,
        tile_aspect_ratio: (u32, u32),
        output_resolution: Resolution,
    ) -> RowsCols {
        let mut best_rows_cols = RowsCols::from_rows_count(inputs_count, 1);
        let mut best_tile_width = 0;

        for rows in 1..=inputs_count {
            let rows_cols = RowsCols::from_rows_count(inputs_count, rows);
            // larger width <=> larger tile size, because of const tile aspect ratio
            let tile_size = Self::tile_size(&rows_cols, tile_aspect_ratio, output_resolution).width;

            if tile_size > best_tile_width {
                best_rows_cols = rows_cols;
                best_tile_width = tile_size;
            }
        }

        best_rows_cols
    }

    fn transformation_matrix(
        tile_layout: &BoxLayout,
        input_resolution: Resolution,
        output_resolution: Resolution,
        tile_size: Resolution,
    ) -> Mat4 {
        let fit = FitParams::new(input_resolution, tile_size).scale_matrix;
        tile_layout.transformation_matrix(output_resolution) * fit
    }
}

fn ceil_div(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}
