use compositor_common::{scene::Resolution, util::colors::RGBAColor};

use crate::{
    scene::ViewChildrenDirection,
    transformations::layout::{Layout, LayoutContent, NestedLayout},
};

use super::{
    components::ViewComponent, BaseNode, BuildSceneError, Component, LayoutComponent, Position,
};

#[derive(Debug)]
struct StaticChildLayoutOpts {
    width: Option<usize>,
    height: Option<usize>,
    /// offset inside parent component
    static_offset: usize,
    /// For direction=row defines width of a dynamically sized component
    /// For direction=column defines height of a dynamically sized component
    dynamic_child_size: usize,
    parent_size: Resolution,
}

impl ViewComponent {
    pub(super) fn base_node(&self) -> Result<BaseNode, BuildSceneError> {
        let children = self
            .children
            .iter()
            .map(|component| {
                let node = component.base_node()?;
                match node {
                    BaseNode::Layout { root: _, children } => Ok(children),
                    _ => Ok(vec![node]),
                }
            })
            .collect::<Result<Vec<_>, _>>()? // convert Vec<Result<Vec<_>>> into Result<Vec<Vec<_>>>
            .into_iter()
            .flatten()
            .collect();

        Ok(BaseNode::Layout {
            root: LayoutComponent::View(self.clone()),
            children,
        })
    }

    pub(super) fn children(&self) -> Vec<&Component> {
        self.children.iter().collect()
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut Component> {
        self.children.iter_mut().collect()
    }

    pub(super) fn layout(&self, size: Resolution) -> Vec<NestedLayout> {
        let dynamic_child_size = self.dynamic_size_for_static_children(size);

        // offset along x or y direction (depends on self.direction) where next
        // child component should be placed
        let mut static_offset = 0;

        self.children
            .iter()
            .map(|child| {
                let position = match child {
                    Component::Layout(layout) => layout.position(),
                    non_layout_component => Position::Static {
                        width: non_layout_component.width(),
                        height: non_layout_component.height(),
                    },
                };
                match position {
                    Position::Static { width, height } => {
                        let (layout, updated_static_offset) = self.layout_static_child(
                            child,
                            StaticChildLayoutOpts {
                                width,
                                height,
                                static_offset,
                                dynamic_child_size,
                                parent_size: size,
                            },
                        );

                        static_offset = updated_static_offset;
                        layout
                    }
                    Position::Relative(position) => {
                        LayoutComponent::layout_relative_child(child, position, size)
                    }
                }
            })
            .collect()
    }

    fn layout_static_child(
        &self,
        child: &Component,
        opts: StaticChildLayoutOpts,
    ) -> (NestedLayout, usize) {
        let mut static_offset = opts.static_offset;
        let (top, left, width, height) = match self.direction {
            ViewChildrenDirection::Row => {
                let width = opts.width.unwrap_or(opts.dynamic_child_size);
                let height = opts.height.unwrap_or(opts.parent_size.height);
                let top = 0.0;
                let left = static_offset;
                static_offset += width;
                (top as f32, left as f32, width as f32, height as f32)
            }
            ViewChildrenDirection::Column => {
                let height = opts.height.unwrap_or(opts.dynamic_child_size);
                let width = opts.width.unwrap_or(opts.parent_size.width);
                let top = static_offset;
                let left = 0.0;
                static_offset += height;
                (top as f32, left as f32, width as f32, height as f32)
            }
        };
        let layout = match child {
            Component::Layout(layout_component) => {
                let children_layouts = layout_component.layout(Resolution {
                    width: width as usize,
                    height: height as usize,
                });
                NestedLayout {
                    layout: Layout {
                        top,
                        left,
                        width,
                        height,
                        rotation_degrees: 0.0,
                        content: LayoutContent::Color(
                            layout_component
                                .background_color()
                                .unwrap_or(RGBAColor(0, 0, 0, 0)),
                        ),
                    },
                    children: children_layouts,
                }
            }
            _ => NestedLayout {
                layout: Layout {
                    top,
                    left,
                    width,
                    height,
                    rotation_degrees: 0.0,
                    content: LayoutContent::ChildNode(0),
                },
                children: vec![],
            },
        };
        (layout, static_offset)
    }

    /// Calculate size of a dynamically sized child component. Returned value
    /// represents width if the direction is `ViewChildrenDirection::Row` or height if
    /// the direction is `ViewChildrenDirection::Column`.
    fn dynamic_size_for_static_children(&self, size: Resolution) -> usize {
        let max_size = match self.direction {
            super::ViewChildrenDirection::Row => size.width,
            super::ViewChildrenDirection::Column => size.height,
        };

        let dynamic_children_count = self
            .static_children_iter()
            .filter(|child| match self.direction {
                ViewChildrenDirection::Row => child.width().is_none(),
                ViewChildrenDirection::Column => child.height().is_none(),
            })
            .count();
        let static_children_sum = self.sum_static_children_sizes();

        if dynamic_children_count == 0 {
            return 0; // if there is no dynamically sized children then this value does not matter
        }
        f32::max(
            0.0,
            (max_size as f32 - static_children_sum as f32) / dynamic_children_count as f32,
        ) as usize
    }

    fn sum_static_children_sizes(&self) -> usize {
        let size_accessor = match self.direction {
            super::ViewChildrenDirection::Row => |component: &Component| component.width(),
            super::ViewChildrenDirection::Column => |component: &Component| component.height(),
        };

        self.static_children_iter()
            .map(|component| size_accessor(component).unwrap_or(0))
            .sum()
    }

    fn static_children_iter(&self) -> impl Iterator<Item = &Component> {
        self.children.iter().filter(|child| match child {
            Component::Layout(layout) => match layout.position() {
                super::Position::Static { .. } => true,
                super::Position::Relative(_) => false,
            },
            _ => true,
        })
    }
}
