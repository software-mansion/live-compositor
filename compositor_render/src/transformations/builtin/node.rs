use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use compositor_common::scene::{builtin_transformations::BuiltinSpec, NodeId, Resolution};
use wgpu::util::DeviceExt;

use crate::{renderer::texture::NodeTexture, transformations::shader::Shader};

use super::{params::BuiltinParams, transformations::BuiltinTransformations};

struct ConstructedBuiltinNode {
    shader: Arc<Shader>,
    spec: BuiltinSpec,
}

struct ConfiguredBuiltinNode {
    shader: Arc<Shader>,
    spec: BuiltinSpec,
    params_bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    input_resolutions: Vec<Option<Resolution>>,
    output_resolution: Resolution,
    clear_color: Option<wgpu::Color>,
}

impl ConfiguredBuiltinNode {
    fn new(
        constructed: &ConstructedBuiltinNode,
        input_resolutions: Vec<Option<Resolution>>,
    ) -> Self {
        let output_resolution =
            BuiltinTransformations::output_resolution(&constructed.spec, &input_resolutions);
        let params = BuiltinParams::new(&constructed.spec, &input_resolutions);
        let wgpu_ctx = constructed.shader.wgpu_ctx.clone();

        let params_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("builtin node params buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: &params.shader_buffer_content(),
            });

        let params_bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("builtin node params bind group"),
                layout: &wgpu_ctx.shader_parameters_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu_ctx
                            .compositor_provided_parameters_buffer
                            .as_entire_binding(),
                    },
                ],
            });

        let clear_color = BuiltinTransformations::clear_color(&constructed.spec);

        Self {
            shader: constructed.shader.clone(),
            spec: constructed.spec.clone(),
            params_bind_group,
            params_buffer,
            input_resolutions,
            output_resolution,
            clear_color,
        }
    }

    fn ensure_configured(&mut self, input_resolutions: Vec<Option<Resolution>>) {
        if self.input_resolutions != input_resolutions {
            let params = BuiltinParams::new(&self.spec, &input_resolutions);
            self.shader.wgpu_ctx.queue.write_buffer(
                &self.params_buffer,
                0,
                &params.shader_buffer_content(),
            );
            self.input_resolutions = input_resolutions;
        }
    }
}

enum BuiltinNodeState {
    Constructed(ConstructedBuiltinNode),
    Configured(ConfiguredBuiltinNode),
}

impl BuiltinNodeState {
    fn new(shader: Arc<Shader>, spec: BuiltinSpec) -> Self {
        Self::Constructed(ConstructedBuiltinNode { shader, spec })
    }

    fn ensure_configured(&mut self, input_resolutions: Vec<Option<Resolution>>) {
        match self {
            BuiltinNodeState::Constructed(ref constructed) => {
                let configured = ConfiguredBuiltinNode::new(constructed, input_resolutions);
                *self = BuiltinNodeState::Configured(configured);
            }
            BuiltinNodeState::Configured(configured) => {
                configured.ensure_configured(input_resolutions);
            }
        };
    }
}

pub struct BuiltinNode(Arc<Mutex<BuiltinNodeState>>);

impl BuiltinNode {
    pub fn new(shader: Arc<Shader>, spec: BuiltinSpec) -> Self {
        Self(Arc::new(Mutex::new(BuiltinNodeState::new(shader, spec))))
    }

    pub fn resolution(&self) -> Option<Resolution> {
        match &*self.0.lock().unwrap() {
            BuiltinNodeState::Constructed(constructed) => match constructed.spec {
                BuiltinSpec::TransformToResolution { resolution, .. } => Some(resolution),
                BuiltinSpec::FixedPositionLayout { resolution, .. } => Some(resolution),
            },
            BuiltinNodeState::Configured(configured) => Some(configured.output_resolution),
        }
    }

    pub fn render(
        &self,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        // TODO: think about fallbacks

        let input_resolutions: Vec<Option<Resolution>> = sources
            .iter()
            .map(|(_, node_texture)| node_texture.resolution())
            .collect();

        let mut guard = self.0.lock().unwrap();

        guard.ensure_configured(input_resolutions);

        // TODO: figure out cleaner way to do this
        let BuiltinNodeState::Configured(configured) = &*guard else {
            panic!("Builtin node state should be configured before render!");
        };

        let shader = configured.shader.clone();

        let target = target.ensure_size(&shader.wgpu_ctx, configured.output_resolution);
        shader.render(
            &configured.params_bind_group,
            sources,
            target,
            pts,
            configured.clear_color,
        );
    }
}
