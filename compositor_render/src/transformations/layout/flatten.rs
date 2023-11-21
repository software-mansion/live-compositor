use compositor_common::util::colors::RGBAColor;

use super::{Crop, LayoutContent, NestedLayout, RenderLayout, RenderLayoutContent};

impl NestedLayout {
    pub(super) fn flatten(mut self, child_index_offset: usize) -> Vec<RenderLayout> {
        let mut child_index_offset = child_index_offset;
        if let LayoutContent::ChildNode { index, size } = self.content {
            self.content = LayoutContent::ChildNode {
                index: index + child_index_offset,
                size,
            };
            child_index_offset += 1
        }
        let layout = self.render_layout();
        let children: Vec<_> = std::mem::take(&mut self.children)
            .into_iter()
            .flat_map(|child| {
                let child_nodes_count = child.child_nodes_count;
                let layouts = child.flatten(child_index_offset);
                child_index_offset += child_nodes_count;
                layouts
            })
            .map(|l| self.flatten_child(l))
            .filter(|layout| {
                !matches!(
                    layout.content,
                    RenderLayoutContent::Color(RGBAColor(0, 0, 0, 0))
                )
            })
            .collect();
        [vec![layout], children].concat()
    }

    fn flatten_child(&self, layout: RenderLayout) -> RenderLayout {
        match &self.crop {
            None => RenderLayout {
                top: self.top + (layout.top * self.scale_y),
                left: self.left + (layout.left * self.scale_x),
                width: layout.width * self.scale_x,
                height: layout.height * self.scale_y,
                rotation_degrees: layout.rotation_degrees + self.rotation_degrees, // TODO: not exactly correct
                content: layout.content,
            },
            Some(crop) => {
                // Bellow values are only correct if `crop` is in the same coordinate
                // system as self.top/self.left/self.width/self.height. This condition
                // will always be fulfilled as long NestedLayout with LayoutContent::ChildNode
                // does not have any child layouts.

                // Value in coordinates of `self` (relative to it's top-left corner). Represents
                // a position after cropping and translated back to (layout.top, layout.left).
                let cropped_top = f32::max(layout.top - crop.top, 0.0);
                let cropped_left = f32::max(layout.left - crop.left, 0.0);
                let cropped_bottom = f32::min(layout.top + layout.height - crop.top, crop.height);
                let cropped_right = f32::min(layout.left + layout.width - crop.left, crop.width);
                let cropped_width = cropped_right - cropped_left;
                let cropped_height = cropped_bottom - cropped_top;
                match layout.content {
                    RenderLayoutContent::Color(color) => {
                        RenderLayout {
                            top: self.top + (cropped_top * self.scale_y),
                            left: self.left + (cropped_left * self.scale_x),
                            width: cropped_width * self.scale_x,
                            height: cropped_height * self.scale_y,
                            rotation_degrees: layout.rotation_degrees + self.rotation_degrees, // TODO: not exactly correct
                            content: RenderLayoutContent::Color(color),
                        }
                    }
                    RenderLayoutContent::ChildNode {
                        index,
                        crop: child_crop,
                    } => {
                        // Calculate how much top/left coordinates changed when cropping. It represents
                        // how much was removed in layout coordinates. Ignore the change of a position that
                        // was a result of a translation after cropping.
                        let top_diff = f32::max(crop.top - layout.top, 0.0);
                        let left_diff = f32::max(crop.left - layout.left, 0.0);

                        // Factor to translate from `layout` coordinates to child node coord.
                        // The same factor holds for translations from `self.layout`.
                        let horizontal_scale_factor = child_crop.width / layout.width;
                        let vertical_scale_factor = child_crop.height / layout.height;

                        let crop = Crop {
                            top: child_crop.top + (top_diff * vertical_scale_factor),
                            left: child_crop.left + (left_diff * horizontal_scale_factor),
                            width: cropped_width * horizontal_scale_factor,
                            height: cropped_height * vertical_scale_factor,
                        };

                        RenderLayout {
                            top: self.top + (cropped_top * self.scale_y),
                            left: self.left + (cropped_left * self.scale_x),
                            width: cropped_width * self.scale_x,
                            height: cropped_height * self.scale_y,
                            rotation_degrees: layout.rotation_degrees + self.rotation_degrees, // TODO: not exactly correct
                            content: RenderLayoutContent::ChildNode { index, crop },
                        }
                    }
                }
            }
        }
    }

    fn render_layout(&self) -> RenderLayout {
        RenderLayout {
            top: self.top,
            left: self.left,
            width: self.width,
            height: self.height,
            rotation_degrees: self.rotation_degrees,
            content: match self.content {
                LayoutContent::Color(color) => RenderLayoutContent::Color(color),
                LayoutContent::ChildNode { index, size } => RenderLayoutContent::ChildNode {
                    index,
                    crop: Crop {
                        top: 0.0,
                        left: 0.0,
                        width: size.width,
                        height: size.height,
                    },
                },
                LayoutContent::None => RenderLayoutContent::Color(RGBAColor(0, 0, 0, 0)),
            },
        }
    }
}
