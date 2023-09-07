use std::vec;

use compositor_common::scene::Resolution;

use nalgebra_glm::{scale, translate, vec3, Mat4};

use super::transform_to_resolution::FitParams;

#[derive(Debug)]
struct Tile {
    top_left_corner: (u32, u32),
    width: u32,
    height: u32,
}

impl Tile {
    fn resolution(&self) -> Resolution {
        Resolution {
            width: self.width as usize,
            height: self.height as usize,
        }
    }
}

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
pub struct GridParams {
    transformation_matrices: Vec<Mat4>,
}

impl GridParams {
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
                Self::texture_transformation_matrix(
                    tile_layout,
                    input_resolution,
                    output_resolution,
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
    ) -> Vec<Tile> {
        let mut layouts = Vec::with_capacity(inputs_count as usize);

        let tiles_width = tile_size.width as u32 * rows_cols.cols;
        let x_padding = (output_resolution.width as u32 - tiles_width) / 2;

        let tiles_height = tile_size.height as u32 * rows_cols.rows;
        let y_padding = (output_resolution.height as u32 - tiles_height) / 2;

        for row in 0..(rows_cols.rows - 1) {
            for col in 0..rows_cols.cols {
                let top_left_corner = (
                    x_padding + col * tile_size.width as u32,
                    y_padding + row * tile_size.height as u32,
                );
                layouts.push(Tile {
                    top_left_corner,
                    width: tile_size.width as u32,
                    height: tile_size.height as u32,
                })
            }
        }

        let bottom_row_tiles_count = inputs_count - ((rows_cols.rows - 1) * rows_cols.cols);

        let bottom_tiles_width = tile_size.width as u32 * bottom_row_tiles_count;
        let bottom_row_x_padding =
            (output_resolution.width as u32 - bottom_tiles_width) / (bottom_row_tiles_count + 1);

        for bottom_tile in 0..bottom_row_tiles_count {
            let top_left_corner = (
                (bottom_row_x_padding + tile_size.width as u32) * bottom_tile
                    + bottom_row_x_padding,
                (rows_cols.rows - 1) * tile_size.height as u32 + y_padding,
            );

            layouts.push(Tile {
                top_left_corner,
                width: tile_size.width as u32,
                height: tile_size.height as u32,
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

    fn texture_transformation_matrix(
        tile_layout: &Tile,
        input_resolution: Resolution,
        output_resolution: Resolution,
    ) -> Mat4 {
        let mut transformation_matrix = Mat4::identity();

        let tile_center_pixels = (
            tile_layout.top_left_corner.0 + (tile_layout.width / 2),
            tile_layout.top_left_corner.1 + (tile_layout.height / 2),
        );

        let tile_center_clip_space = (
            interpolate_x(tile_center_pixels.0 as f32, output_resolution.width as f32),
            interpolate_y(tile_center_pixels.1 as f32, output_resolution.height as f32),
        );

        transformation_matrix = translate(
            &transformation_matrix,
            &vec3(tile_center_clip_space.0, tile_center_clip_space.1, 0.0),
        );

        let fit_params = FitParams::new(input_resolution, tile_layout.resolution());
        let scale_to_tile_resolution = (
            tile_layout.width as f32 / output_resolution.width as f32,
            tile_layout.height as f32 / output_resolution.height as f32,
        );

        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(
                fit_params.x_scale * scale_to_tile_resolution.0,
                fit_params.y_scale * scale_to_tile_resolution.1,
                1.0,
            ),
        );

        transformation_matrix
    }

    /// Optimize number of rows and cols to maximize space covered by tiles,
    /// preserving tile aspect_ratio
    fn optimize_inputs_layout(
        inputs_count: u32,
        tile_aspect_ratio: (u32, u32),
        output_resolution: Resolution,
    ) -> RowsCols {
        fn tiles_scale(
            rows_cols: &RowsCols,
            tile_aspect_ratio: (u32, u32),
            output_resolution: Resolution,
        ) -> f32 {
            let x_scale =
                output_resolution.width as f32 / rows_cols.cols as f32 / tile_aspect_ratio.0 as f32;
            let y_scale = output_resolution.height as f32
                / rows_cols.rows as f32
                / tile_aspect_ratio.1 as f32;

            if x_scale < y_scale {
                x_scale
            } else {
                y_scale
            }
        }

        let mut best_row_cols = RowsCols::from_rows_count(inputs_count, 1);
        let mut best_tiles_scale = 0.0;

        for rows in 1..inputs_count {
            let row_cols = RowsCols::from_rows_count(inputs_count, rows);
            let tiles_scale = tiles_scale(&row_cols, tile_aspect_ratio, output_resolution);

            if tiles_scale > best_tiles_scale {
                best_row_cols = row_cols;
                best_tiles_scale = tiles_scale;
            }
        }

        best_row_cols
    }
}

fn ceil_div(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

fn interpolate_x(x: f32, output_width: f32) -> f32 {
    x / output_width * 2.0 - 1.0
}

fn interpolate_y(y: f32, output_height: f32) -> f32 {
    1.0 - (y / output_height * 2.0)
}
