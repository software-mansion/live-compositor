use std::time::Duration;

use crate::{
    scene::{
        layout::StatefulLayoutComponent, HorizontalAlign, RescaleMode, Size, StatefulComponent,
        VerticalAlign,
    },
    transformations::layout::{Crop, LayoutContent, NestedLayout},
};

use super::RescalerComponentParam;

impl RescalerComponentParam {
    pub(super) fn layout(
        &self,
        size: Size,
        child: &StatefulComponent,
        pts: Duration,
    ) -> NestedLayout {
        let child_width = child.width(pts);
        let child_height = child.height(pts);
        match (child_width, child_height) {
            (None, None) => self.layout_with_scale(size, child, pts, 1.0),
            (None, Some(child_height)) => {
                self.layout_with_scale(size, child, pts, size.height / child_height)
            }
            (Some(child_width), None) => {
                self.layout_with_scale(size, child, pts, size.width / child_width)
            }
            (Some(child_width), Some(child_height)) => {
                let scale = match self.mode {
                    RescaleMode::Fit => {
                        f32::min(size.width / child_width, size.height / child_height)
                    }
                    RescaleMode::Fill => {
                        f32::max(size.width / child_width, size.height / child_height)
                    }
                };
                self.layout_with_scale(size, child, pts, scale)
            }
        }
    }

    fn layout_with_scale(
        &self,
        size: Size,
        child: &StatefulComponent,
        pts: Duration,
        scale: f32,
    ) -> NestedLayout {
        let (content, children, child_nodes_count) = match child {
            StatefulComponent::Layout(layout_component) => {
                let children_layouts = layout_component.layout(
                    Size {
                        width: size.width / scale,
                        height: size.height / scale,
                    },
                    pts,
                );
                let child_nodes_count = children_layouts.child_nodes_count;
                (
                    LayoutContent::None,
                    vec![children_layouts],
                    child_nodes_count,
                )
            }
            _non_layout => (StatefulLayoutComponent::layout_content(child, 0), vec![], 1),
        };

        let top = match self.vertical_align {
            VerticalAlign::Top => 0.0,
            VerticalAlign::Bottom => child
                .height(pts)
                .map(|height| size.height - (height * scale))
                .unwrap_or(0.0),
            VerticalAlign::Center | VerticalAlign::Justified => child
                .height(pts)
                .map(|height| (size.height - (height * scale)) / 2.0)
                .unwrap_or(0.0),
        };
        let left = match self.horizontal_align {
            HorizontalAlign::Left => 0.0,
            HorizontalAlign::Right => child
                .width(pts)
                .map(|width| (size.width - (width * scale)))
                .unwrap_or(0.0),
            HorizontalAlign::Center | HorizontalAlign::Justified => child
                .width(pts)
                .map(|width| (size.width - (width * scale)) / (2.0))
                .unwrap_or(0.0),
        };

        let width = child
            .width(pts)
            .map(|child_width| child_width * scale)
            .unwrap_or(size.width);
        let height = child
            .height(pts)
            .map(|child_height| child_height * scale)
            .unwrap_or(size.height);

        NestedLayout {
            top: 0.0,
            left: 0.0,
            width: size.width,
            height: size.height,
            rotation_degrees: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            crop: Some(Crop {
                top: 0.0,
                left: 0.0,
                width: size.width,
                height: size.height,
            }),
            content: LayoutContent::None,
            children: vec![NestedLayout {
                top,
                left,
                width,
                height,
                rotation_degrees: 0.0,
                scale_x: scale,
                scale_y: scale,
                crop: None,
                content,
                child_nodes_count,
                children,
            }],
            child_nodes_count,
        }
    }
}
