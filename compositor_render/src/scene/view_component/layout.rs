use std::time::Duration;

use compositor_common::scene::Resolution;

use crate::{
    scene::{layout::LayoutComponentState, ComponentState, Position, ViewChildrenDirection},
    transformations::layout::{Layout, LayoutContent, NestedLayout},
};

use super::ViewComponentParam;

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

impl ViewComponentParam {
    pub(super) fn layout(
        &self,
        size: Resolution,
        children: &[ComponentState],
        pts: Duration,
    ) -> NestedLayout {
        let dynamic_child_size = self.dynamic_size_for_static_children(size, children, pts);

        // offset along x or y direction (depends on self.direction) where next
        // child component should be placed
        let mut static_offset = 0;

        let children: Vec<_> = children
            .iter()
            .map(|child| {
                let position = match child {
                    ComponentState::Layout(layout) => layout.position(pts),
                    non_layout_component => Position::Static {
                        width: non_layout_component.width(pts),
                        height: non_layout_component.height(pts),
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
                            pts,
                        );

                        static_offset = updated_static_offset;
                        layout
                    }
                    Position::Relative(position) => {
                        LayoutComponentState::layout_relative_child(child, position, size, pts)
                    }
                }
            })
            .collect();
        NestedLayout {
            layout: Layout {
                top: 0.0,
                left: 0.0,
                width: size.width as f32,
                height: size.height as f32,
                rotation_degrees: 0.0,
                content: LayoutContent::Color(self.background_color),
            },
            child_nodes_count: children.iter().map(|l| l.child_nodes_count).sum(),
            children,
        }
    }

    fn layout_static_child(
        &self,
        child: &ComponentState,
        opts: StaticChildLayoutOpts,
        pts: Duration,
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
            ComponentState::Layout(layout_component) => {
                let children_layouts = layout_component.layout(
                    Resolution {
                        width: width as usize,
                        height: height as usize,
                    },
                    pts,
                );
                NestedLayout {
                    layout: Layout {
                        top,
                        left,
                        width,
                        height,
                        rotation_degrees: 0.0,
                        content: LayoutContent::None,
                    },
                    child_nodes_count: children_layouts.child_nodes_count,
                    children: vec![children_layouts],
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
                child_nodes_count: 1,
                children: vec![],
            },
        };
        (layout, static_offset)
    }

    /// Calculate size of a dynamically sized child component. Returned value
    /// represents width if the direction is `ViewChildrenDirection::Row` or height if
    /// the direction is `ViewChildrenDirection::Column`.
    fn dynamic_size_for_static_children(
        &self,
        size: Resolution,
        children: &[ComponentState],
        pts: Duration,
    ) -> usize {
        let max_size = match self.direction {
            super::ViewChildrenDirection::Row => size.width,
            super::ViewChildrenDirection::Column => size.height,
        };

        let dynamic_children_count = Self::static_children_iter(children, pts)
            .filter(|child| match self.direction {
                ViewChildrenDirection::Row => child.width(pts).is_none(),
                ViewChildrenDirection::Column => child.height(pts).is_none(),
            })
            .count();
        let static_children_sum = self.sum_static_children_sizes(children, pts);

        if dynamic_children_count == 0 {
            return 0; // if there is no dynamically sized children then this value does not matter
        }
        f32::max(
            0.0,
            (max_size as f32 - static_children_sum as f32) / dynamic_children_count as f32,
        ) as usize
    }

    fn sum_static_children_sizes(&self, children: &[ComponentState], pts: Duration) -> usize {
        let size_accessor = match self.direction {
            super::ViewChildrenDirection::Row => ComponentState::width,
            super::ViewChildrenDirection::Column => ComponentState::height,
        };

        Self::static_children_iter(children, pts)
            .map(|component| size_accessor(component, pts).unwrap_or(0))
            .sum()
    }

    fn static_children_iter(
        children: &[ComponentState],
        pts: Duration,
    ) -> impl Iterator<Item = &ComponentState> {
        children.iter().filter(move |child| match child {
            ComponentState::Layout(layout) => match layout.position(pts) {
                super::Position::Static { .. } => true,
                super::Position::Relative(_) => false,
            },
            _ => true,
        })
    }
}
