use std::time::Duration;

use crate::{
    scene::{
        layout::StatefulLayoutComponent, HorizontalAlign, Size, StatefulComponent, VerticalAlign,
    },
    transformations::layout::{LayoutContent, NestedLayout},
};

use super::TilesComponentParams;

#[derive(Debug, Clone, Copy)]
struct RowsCols {
    rows: u32,
    columns: u32,
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    top: f32,
    left: f32,
    width: f32,
    height: f32,
}

impl TilesComponentParams {
    pub(super) fn layout(
        &self,
        size: Size,
        children: &[StatefulComponent],
        pts: Duration,
    ) -> NestedLayout {
        let input_count = children.len() as u32;
        let rows_cols = self.optimal_row_column_count(input_count, size);
        let tile_size = self.tile_size(rows_cols, size);
        let tiles = self.tiles_positions(input_count, rows_cols, tile_size, size);
        let children = children
            .iter()
            .zip(tiles)
            .map(|(component, tile)| Self::layout_child(component, tile, pts))
            .collect::<Vec<_>>();

        NestedLayout {
            top: 0.0,
            left: 0.0,
            width: size.width,
            height: size.height,
            rotation_degrees: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            crop: None,
            content: LayoutContent::Color(self.background_color),
            child_nodes_count: children.iter().map(|l| l.child_nodes_count).sum(),
            children,
        }
    }

    fn layout_child(child: &StatefulComponent, tile: Tile, pts: Duration) -> NestedLayout {
        match child {
            StatefulComponent::Layout(layout_component) => {
                let children_layouts = layout_component.layout(
                    Size {
                        width: tile.width,
                        height: tile.height,
                    },
                    pts,
                );
                NestedLayout {
                    top: tile.top,
                    left: tile.left,
                    width: tile.width,
                    height: tile.height,
                    rotation_degrees: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    crop: None,
                    content: LayoutContent::None,
                    child_nodes_count: children_layouts.child_nodes_count,
                    children: vec![children_layouts],
                }
            }
            _ => {
                let fitted = fit_into_tile(tile, child, pts);
                NestedLayout {
                    // TODO: fit
                    top: fitted.top,
                    left: fitted.left,
                    width: fitted.width,
                    height: fitted.height,
                    rotation_degrees: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    crop: None,
                    content: StatefulLayoutComponent::layout_content(child, 0),
                    child_nodes_count: 1,
                    children: vec![],
                }
            }
        }
    }

    /// Optimize number of rows and cols to maximize space covered by tiles,
    /// preserving tile aspect_ratio
    fn optimal_row_column_count(&self, inputs_count: u32, layout_size: Size) -> RowsCols {
        fn from_rows_count(inputs_count: u32, rows: u32) -> RowsCols {
            let columns = (inputs_count + rows - 1) / rows;
            RowsCols { rows, columns }
        }
        let mut best_rows_cols = from_rows_count(inputs_count, 1);
        let mut best_tile_width = 0.0;

        for rows in 1..=inputs_count {
            let rows_cols = from_rows_count(inputs_count, rows);
            // larger width <=> larger tile size, because of const tile aspect ratio
            let tile_size = self.tile_size(rows_cols, layout_size).width;
            if tile_size > best_tile_width {
                best_rows_cols = rows_cols;
                best_tile_width = tile_size;
            }
        }

        best_rows_cols
    }

    fn tile_size(&self, rows_cols: RowsCols, layout_size: Size) -> Size {
        let x_padding = rows_cols.columns as f32 * 2.0 * self.padding;
        let y_padding = rows_cols.rows as f32 * 2.0 * self.padding;
        let x_margin = (rows_cols.columns as f32 + 1.0) * self.margin;
        let y_margin = (rows_cols.rows as f32 + 1.0) * self.margin;

        let x_scale = (layout_size.width - x_padding - x_margin).max(0.0)
            / rows_cols.columns as f32
            / self.tile_aspect_ratio.0 as f32;
        let y_scale = (layout_size.height - y_padding - y_margin).max(0.0)
            / rows_cols.rows as f32
            / self.tile_aspect_ratio.1 as f32;

        let scale = if x_scale < y_scale { x_scale } else { y_scale };

        Size {
            width: self.tile_aspect_ratio.0 as f32 * scale,
            height: self.tile_aspect_ratio.1 as f32 * scale,
        }
    }

    fn tiles_positions(
        &self,
        inputs_count: u32,
        rows_cols: RowsCols,
        tile_size: Size,
        layout_size: Size,
    ) -> Vec<Tile> {
        let mut layouts = Vec::with_capacity(inputs_count as usize);

        // Because scaled tiles with padding and margin don't have to cover whole output frame,
        // additional padding is distributed is distributed accordingly to alignment
        let additional_y_padding = layout_size.height
            - (tile_size.height + 2.0 * self.padding) * rows_cols.rows as f32
            - (self.margin * (rows_cols.rows as f32 + 1.0));

        let (additional_top_padding, justified_padding_y) = match self.vertical_align {
            VerticalAlign::Top => (0.0, 0.0),
            VerticalAlign::Center => (additional_y_padding / 2.0, 0.0),
            VerticalAlign::Bottom => (additional_y_padding, 0.0),
            VerticalAlign::Justified => {
                let space = additional_y_padding / (rows_cols.rows as f32 + 1.0);
                (0.0, space)
            }
        };

        let mut top = additional_top_padding + justified_padding_y + self.padding + self.margin;
        for row in 0..rows_cols.rows {
            let tiles_in_row = if row < rows_cols.rows - 1 {
                rows_cols.columns
            } else {
                inputs_count - ((rows_cols.rows - 1) * rows_cols.columns)
            };

            let additional_x_padding = layout_size.width
                - (tile_size.width + 2.0 * self.padding) * tiles_in_row as f32
                - (self.margin * (tiles_in_row as f32 + 1.0));

            let (additional_left_padding, justified_padding_x) = match self.horizontal_align {
                HorizontalAlign::Left => (0.0, 0.0),
                HorizontalAlign::Right => (additional_x_padding, 0.0),
                HorizontalAlign::Justified => {
                    let space = additional_x_padding / (tiles_in_row + 1) as f32;
                    (0.0, space)
                }
                HorizontalAlign::Center => (additional_x_padding / 2.0, 0.0),
            };

            let mut left =
                additional_left_padding + justified_padding_x + self.margin + self.padding;

            for _col in 0..tiles_in_row {
                layouts.push(Tile {
                    top,
                    left,
                    width: tile_size.width,
                    height: tile_size.height,
                });

                left += tile_size.width + self.margin + self.padding * 2.0 + justified_padding_x;
            }
            top += tile_size.height + self.margin + self.padding * 2.0 + justified_padding_y;
        }

        layouts
    }
}

fn fit_into_tile(tile: Tile, component: &StatefulComponent, pts: Duration) -> Tile {
    let Some(width) = component.width(pts) else {
        return tile;
    };
    let Some(height) = component.height(pts) else {
        return tile;
    };
    let scale_to_fit_width = tile.width / width;
    let scale_to_fit_height = tile.height / height;
    let scale_factor = f32::min(scale_to_fit_width, scale_to_fit_height);

    let top_offset = (tile.height - scale_factor * height) / 2.0;
    let left_offset = (tile.width - scale_factor * width) / 2.0;

    Tile {
        top: tile.top + top_offset,
        left: tile.left + left_offset,
        width: scale_factor * width,
        height: scale_factor * height,
    }
}
