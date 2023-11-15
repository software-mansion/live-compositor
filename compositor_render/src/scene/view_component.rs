use compositor_common::{scene::Resolution, util::colors::RGBAColor};

use crate::{
    scene::ViewChildrenDirection,
    transformations::layout::{Layout, LayoutContent, NestedLayout},
};

use super::{components::ViewComponent, BaseNode, BuildSceneError, Component, LayoutComponent};

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

        let create_static_layout = match self.direction {
            ViewChildrenDirection::Row => Self::create_static_layout_row,
            ViewChildrenDirection::Column => Self::create_static_layout_column,
        };

        self.children
            .iter()
            .map(|child| {
                if Self::is_static_child(child) {
                    let layout =
                        create_static_layout(child, static_offset, dynamic_child_size, size);
                    match self.direction {
                        ViewChildrenDirection::Row => static_offset += layout.width as usize,
                        ViewChildrenDirection::Column => static_offset += layout.height as usize,
                    };
                    match child {
                        Component::Layout(layout_component) => {
                            let children_layouts = layout_component.layout(Resolution {
                                width: layout.width as usize,
                                height: layout.height as usize,
                            });
                            NestedLayout {
                                layout: Layout {
                                    content: LayoutContent::Color(
                                        layout_component
                                            .background_color()
                                            .unwrap_or(RGBAColor(0, 0, 0, 0)),
                                    ),
                                    ..layout
                                },
                                children: children_layouts,
                            }
                        }
                        _ => {
                            NestedLayout {
                                layout: layout.clone(),
                                children: vec![NestedLayout {
                                    layout: Layout {
                                        top: 0.0,
                                        left: 0.0,
                                        content: LayoutContent::ChildNode(0), // TODO: this will be recalculated latter
                                        ..layout
                                    },
                                    children: vec![],
                                }],
                            }
                        }
                    }
                } else {
                    // Add support to top/left/right/bottom options
                    todo!()
                }
            })
            .collect()
    }

    fn create_static_layout_row(
        component: &Component,
        offset: usize,
        width_for_dynamic: usize,
        parent_size: Resolution,
    ) -> Layout {
        let width = component.width().unwrap_or(width_for_dynamic);
        Layout {
            top: 0.0,
            left: offset as f32,
            width: width as f32,
            height: component.height().unwrap_or(parent_size.height) as f32,
            rotation_degrees: 0.0,
            content: LayoutContent::Color(RGBAColor(0, 0, 0, 0)),
        }
    }

    fn create_static_layout_column(
        component: &Component,
        offset: usize,
        height_for_dynamic: usize,
        parent_size: Resolution,
    ) -> Layout {
        let height = component.height().unwrap_or(height_for_dynamic);
        Layout {
            top: offset as f32,
            left: 0.0,
            width: component.width().unwrap_or(parent_size.width) as f32,
            height: height as f32,
            rotation_degrees: 0.0,
            content: LayoutContent::Color(RGBAColor(0, 0, 0, 0)),
        }
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
        let static_children_sizes = self.sum_static_children_sizes();

        if dynamic_children_count == 0 {
            return 0; // if there is no dynamically sized children then this value does not matter
        }
        f32::max(
            0.0,
            (max_size as f32 - static_children_sizes as f32) / dynamic_children_count as f32,
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
        self.children.iter().filter(|v| Self::is_static_child(v))
    }

    /// Resolves if specific component will be positioned in the row or column layout. Function
    /// should return false for components that are positioned absolutely inside the parent component.
    fn is_static_child(component: &Component) -> bool {
        match component {
            super::Component::Layout(_layout) => true, // TODO: add support for top/left/right/bottom
            _ => true,
        }
    }
}
