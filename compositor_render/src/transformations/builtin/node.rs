use std::{sync::Arc, time::Duration};

use compositor_common::{
    renderer_spec::FallbackStrategy,
    scene::{
        builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
        NodeId, Resolution,
    },
};

use crate::{
    gpu_shader::GpuShader,
    renderer::{texture::NodeTexture, RenderCtx},
    utils::rgba_to_wgpu_color,
};

use super::{params::RenderParams, shader_params::ParamsBuffer, BuiltinState};

pub struct BuiltinNode {
    state: BuiltinState,
    gpu_shader: Arc<GpuShader>,
    params_buffer: ParamsBuffer,
}

impl BuiltinNode {
    pub fn new_static(ctx: &RenderCtx, spec: &BuiltinSpec, input_count: usize) -> Self {
        Self::new(ctx, BuiltinState::Static(spec.clone()), input_count)
    }

    pub fn new(ctx: &RenderCtx, state: BuiltinState, input_count: usize) -> Self {
        let gpu_shader = ctx.renderers.builtin.gpu_shader(&state);

        let input_resolutions = vec![None; input_count];

        let params_buffer_content =
            RenderParams::new(&state, &input_resolutions).shader_buffer_content();
        let params_buffer = ParamsBuffer::new(params_buffer_content, &gpu_shader.wgpu_ctx);

        Self {
            state,
            gpu_shader,
            params_buffer,
        }
    }

    pub fn state(&self) -> &BuiltinState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut BuiltinState {
        &mut self.state
    }

    pub fn render(
        &mut self,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        let input_resolutions: Vec<Option<Resolution>> = sources
            .iter()
            .map(|(_, node_texture)| node_texture.resolution())
            .collect();

        let output_resolution = self.output_resolution(&input_resolutions);

        let params_buffer_content =
            RenderParams::new(&self.state, &input_resolutions).shader_buffer_content();

        self.params_buffer
            .update(params_buffer_content, &self.gpu_shader.wgpu_ctx);

        let target = target.ensure_size(&self.gpu_shader.wgpu_ctx, output_resolution);
        self.gpu_shader.render(
            self.params_buffer.bind_group(),
            sources,
            target,
            pts,
            self.clear_color(),
        );
    }

    pub fn clear_color(&self) -> Option<wgpu::Color> {
        match &self.state {
            // do not interpolate clear
            BuiltinState::Interpolated { transition, .. } => transition.final_state().clear_color(),
            BuiltinState::Static(spec) => spec.clear_color(),
        }
    }

    pub fn output_resolution(&self, input_resolutions: &[Option<Resolution>]) -> Resolution {
        match &self.state {
            BuiltinState::Interpolated { transition, .. } => transition
                .final_state()
                .output_resolution(input_resolutions),
            BuiltinState::Static(spec) => spec.output_resolution(input_resolutions),
        }
    }

    // Returns Some(Resolution) if output resolution of node can be determined
    // from spec (on scene update). If output resolution is depended on input resolutions,
    // then returns None.
    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        match &self.state {
            BuiltinState::Interpolated { transition, .. } => transition.final_state().resolution(),
            BuiltinState::Static(spec) => spec.resolution(),
        }
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        match &self.state {
            BuiltinState::Interpolated { transition, .. } => {
                transition.final_state().fallback_strategy()
            }
            BuiltinState::Static(spec) => spec.fallback_strategy(),
        }
    }
}

trait BuiltinSpecExt {
    fn clear_color(&self) -> Option<wgpu::Color>;
    fn output_resolution(&self, input_resolutions: &[Option<Resolution>]) -> Resolution;
    fn resolution(&self) -> Option<Resolution>;
    fn fallback_strategy(&self) -> FallbackStrategy;
}

impl BuiltinSpecExt for BuiltinSpec {
    fn clear_color(&self) -> Option<wgpu::Color> {
        match self {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Stretch | TransformToResolutionStrategy::Fill => {
                    None
                }
                TransformToResolutionStrategy::Fit {
                    background_color_rgba,
                    ..
                } => Some(rgba_to_wgpu_color(background_color_rgba)),
            },
            BuiltinSpec::FixedPositionLayout(spec) => {
                Some(rgba_to_wgpu_color(&spec.background_color_rgba))
            }
            BuiltinSpec::TiledLayout(spec) => Some(rgba_to_wgpu_color(&spec.background_color_rgba)),
            BuiltinSpec::CornersRounding { .. } => Some(wgpu::Color::TRANSPARENT),
            BuiltinSpec::MirrorImage { .. } => None,
        }
    }

    fn output_resolution(&self, input_resolutions: &[Option<Resolution>]) -> Resolution {
        fn first_input_resolution(input_resolutions: &[Option<Resolution>]) -> Resolution {
            input_resolutions
                .first()
                .copied()
                .flatten()
                .unwrap_or(Resolution {
                    width: 1,
                    height: 1,
                })
        }

        match &self {
            BuiltinSpec::TransformToResolution { resolution, .. } => *resolution,
            BuiltinSpec::FixedPositionLayout(spec) => spec.resolution,
            BuiltinSpec::TiledLayout(spec) => spec.resolution,
            BuiltinSpec::MirrorImage { .. } => first_input_resolution(input_resolutions),
            BuiltinSpec::CornersRounding { .. } => first_input_resolution(input_resolutions),
        }
    }

    fn resolution(&self) -> Option<Resolution> {
        match self {
            BuiltinSpec::TransformToResolution { resolution, .. } => Some(*resolution),
            BuiltinSpec::FixedPositionLayout(spec) => Some(spec.resolution),
            BuiltinSpec::TiledLayout(spec) => Some(spec.resolution),
            BuiltinSpec::MirrorImage { .. } => None,
            BuiltinSpec::CornersRounding { .. } => None,
        }
    }

    fn fallback_strategy(&self) -> FallbackStrategy {
        match self {
            BuiltinSpec::TransformToResolution { .. }
            | BuiltinSpec::FixedPositionLayout { .. }
            | BuiltinSpec::TiledLayout { .. }
            | BuiltinSpec::MirrorImage { .. }
            | BuiltinSpec::CornersRounding { .. } => FallbackStrategy::FallbackIfAllInputsMissing,
        }
    }
}
