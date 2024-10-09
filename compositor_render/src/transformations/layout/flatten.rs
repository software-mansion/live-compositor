use crate::{scene::RGBAColor, Resolution};

use super::{
    BorderRadius, BoxShadow, Crop, LayoutContent, NestedLayout, ParentBorderRadius, RenderLayout,
    RenderLayoutContent,
};

impl NestedLayout {
    pub(super) fn flatten(
        self,
        input_resolutions: &[Option<Resolution>],
        resolution: Resolution,
    ) -> Vec<RenderLayout> {
        let (shadow, layouts) = self.inner_flatten(0, &vec![]);
        shadow
            .into_iter()
            .chain(layouts.into_iter())
            .filter(|layout| Self::should_render(layout, input_resolutions, resolution))
            .collect()
    }

    fn inner_flatten(
        mut self,
        child_index_offset: usize,
        parent_border_radiuses: &Vec<ParentBorderRadius>,
    ) -> (Vec<RenderLayout>, Vec<RenderLayout>) {
        let mut child_index_offset = child_index_offset;
        if let LayoutContent::ChildNode { index, size } = self.content {
            self.content = LayoutContent::ChildNode {
                index: index + child_index_offset,
                size,
            };
            child_index_offset += 1
        }
        let parent_border_radiuses = recalculate_from_parent(parent_border_radiuses);
        let layout = self.render_layout();
        // It is separated because box shadows of all siblings need to be rendered before
        // this layout and it's siblings
        let box_shadow_layouts = self
            .box_shadow
            .iter()
            .map(|shadow| self.box_shadow_layout(shadow))
            .collect();

        let (children_shadow, children_layouts): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.children)
                .into_iter()
                .map(|child| {
                    let child_nodes_count = child.child_nodes_count;
                    let (shadows, layouts) = child.inner_flatten(child_index_offset);
                    child_index_offset += child_nodes_count;
                    (shadows, layouts)
                })
                .unzip();
        let children_shadow = children_shadow.into_iter().flatten().collect();
        let children_layouts = children_layouts
            .into_iter()
            .flatten()
            .map(|l| self.flatten_child(layout, parent_border_radiuses))
            .collect();

        (
            box_shadow_layouts,
            [vec![layout], children_shadow, children_layouts].concat(),
        )
    }

    fn should_render(
        layout: &RenderLayout,
        input_resolutions: &[Option<Resolution>],
        resolution: Resolution,
    ) -> bool {
        if layout.width <= 0.0
            || layout.height <= 0.0
            || layout.top > resolution.height as f32
            || layout.left > resolution.width as f32
        {
            return false;
        }
        match &layout.content {
            RenderLayoutContent::Color {
                color: RGBAColor(_, _, _, 0),
                border_color,
                border_width,
            } => false,
            RenderLayoutContent::Color { .. } => true,
            RenderLayoutContent::ChildNode {
                crop,
                index,
                border_color,
                border_width,
            } => {
                let size = input_resolutions.get(*index).copied().flatten();
                if let Some(size) = size {
                    if crop.left > size.width as f32 || crop.top > size.height as f32 {
                        return false;
                    }
                }
                if crop.top + crop.height < 0.0 || crop.left + crop.width < 0.0 {
                    return false;
                }
                true
            }
            RenderLayoutContent::BoxShadow { color, blur_radius } => todo!(),
        }
    }

    fn flatten_child(
        &self,
        layout: RenderLayout,
        parent_border_radiuses: &Vec<ParentBorderRadius>,
    ) -> RenderLayout {
        match &self.crop {
            None => RenderLayout {
                top: self.top + (layout.top * self.scale_y),
                left: self.left + (layout.left * self.scale_x),
                width: layout.width * self.scale_x,
                height: layout.height * self.scale_y,
                rotation_degrees: layout.rotation_degrees + self.rotation_degrees, // TODO: not exactly correct
                content: layout.content,
                // TODO: This will not work correctly for layouts that are not proportionally
                // scaled
                border_radius: BorderRadius {
                    top_left: layout.border_radius.top_left * f32::min(self.scale_x, self.scale_y),
                    top_right: layout.border_radius.top_right
                        * f32::min(self.scale_x, self.scale_y),
                    bottom_right: layout.border_radius.bottom_right
                        * f32::min(self.scale_x, self.scale_y),
                    bottom_left: layout.border_radius.bottom_left
                        * f32::min(self.scale_x, self.scale_y),
                },
                parent_border_radiuses,
            },
            Some(crop) => {
                // Below values are only correct if `crop` is in the same coordinate
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
                    RenderLayoutContent::Color {
                        color,
                        border_color,
                        border_width,
                    } => {
                        RenderLayout {
                            top: self.top + (cropped_top * self.scale_y),
                            left: self.left + (cropped_left * self.scale_x),
                            width: cropped_width * self.scale_x,
                            height: cropped_height * self.scale_y,
                            rotation_degrees: layout.rotation_degrees + self.rotation_degrees, // TODO: not exactly correct
                            content: RenderLayoutContent::Color {
                                color,
                                border_color: todo!(),
                                border_width: todo!(),
                            },
                            border_radius: BorderRadius {
                                top_left: todo!(),
                                top_right: todo!(),
                                bottom_right: todo!(),
                                bottom_left: todo!(),
                            },
                            parent_border_radiuses,
                        }
                    }
                    RenderLayoutContent::ChildNode {
                        index,
                        crop: child_crop,
                        border_color,
                        border_width,
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
                            content: RenderLayoutContent::ChildNode {
                                index,
                                crop,
                                border_color: todo!(),
                                border_width: todo!(),
                            },
                            border_radius: todo!(),
                            parent_border_radiuses,
                        }
                    }
                    RenderLayoutContent::BoxShadow { color, blur_radius } => todo!(),
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
                LayoutContent::Color(color) => RenderLayoutContent::Color {
                    color,
                    border_color: todo!(),
                    border_width: todo!(),
                },
                LayoutContent::ChildNode { index, size } => RenderLayoutContent::ChildNode {
                    index,
                    crop: Crop {
                        top: 0.0,
                        left: 0.0,
                        width: size.width,
                        height: size.height,
                    },
                    border_color: todo!(),
                    border_width: todo!(),
                },
                LayoutContent::None => RenderLayoutContent::Color {
                    color: RGBAColor(0, 0, 0, 0),
                    border_color: todo!(),
                    border_width: todo!(),
                },
            },
            border_radius: self.border_radius,
            parent_border_radiuses: todo!(),
        }
    }

    fn box_shadow_layout(&self, box_shadow: &BoxShadow) -> RenderLayout {
        RenderLayout {
            top: self.top + box_shadow.offset_y,
            left: self.left + box_shadow.offset_x,
            width: self.width,
            height: self.height,
            rotation_degrees: self.rotation_degrees, // TODO: this is incorrect
            border_radius: BorderRadius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            parent_border_radiuses: vec![],
            content: RenderLayoutContent::BoxShadow {
                color: box_shadow.color,
                blur_radius: box_shadow.blur_radius,
            },
        }
    }
}
