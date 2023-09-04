use std::{sync::Arc, time::Duration};

use compositor_common::scene::{NodeId, Resolution};
use wgpu::util::DeviceExt;

use crate::{renderer::texture::NodeTexture, transformations::shader::Shader};

use super::{params::BuiltinParams, Builtin};

pub struct ConstructedBuiltinNode {
    shader: Arc<Shader>,
    builtin: Builtin,
}

pub struct ConfiguredBuiltinNode {
    shader: Arc<Shader>,
    builtin: Builtin,
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
        let output_resolution = constructed.builtin.output_resolution(&input_resolutions);
        let params = BuiltinParams::new(&constructed.builtin, &input_resolutions);
        let wgpu_ctx = constructed.shader.wgpu_ctx.clone();

        // This could be created on ConstructedBuiltinNode initialization, but we would need
        // to know size of buffer content
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

        let clear_color = constructed.builtin.clear_color();

        Self {
            shader: constructed.shader.clone(),
            builtin: constructed.builtin.clone(),
            params_bind_group,
            params_buffer,
            input_resolutions,
            output_resolution,
            clear_color,
        }
    }

    fn ensure_configured(&mut self, input_resolutions: Vec<Option<Resolution>>) {
        if self.input_resolutions != input_resolutions {
            let params = BuiltinParams::new(&self.builtin, &input_resolutions);
            self.shader.wgpu_ctx.queue.write_buffer(
                &self.params_buffer,
                0,
                &params.shader_buffer_content(),
            );
            self.input_resolutions = input_resolutions;
        }
    }
}

pub enum BuiltinNode {
    Constructed(ConstructedBuiltinNode),
    Configured(ConfiguredBuiltinNode),
}

impl BuiltinNode {
    pub fn new(shader: Arc<Shader>, spec: Builtin) -> Self {
        Self::Constructed(ConstructedBuiltinNode { shader, builtin: spec })
    }

    // Returns Some(Resolution) if output resolution of node can be determined
    // from spec (on scene update). If output resolution is depended on input resolutions,
    // then returns None.
    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        self.builtin().resolution_from_spec()
    }

    fn builtin(&self) -> &Builtin {
        match &self {
            BuiltinNode::Constructed(ConstructedBuiltinNode { builtin, .. }) => builtin,
            BuiltinNode::Configured(ConfiguredBuiltinNode { builtin, .. }) => builtin,
        }
    }

    pub fn render(
        &mut self,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        // TODO: think about fallbacks

        let input_resolutions: Vec<Option<Resolution>> = sources
            .iter()
            .map(|(_, node_texture)| node_texture.resolution())
            .collect();

        let configured = self.ensure_configured(input_resolutions);

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

    fn ensure_configured(
        &mut self,
        input_resolutions: Vec<Option<Resolution>>,
    ) -> &ConfiguredBuiltinNode {
        match self {
            BuiltinNode::Constructed(ref constructed) => {
                let configured = ConfiguredBuiltinNode::new(constructed, input_resolutions);
                *self = BuiltinNode::Configured(configured);
            }
            BuiltinNode::Configured(configured) => {
                configured.ensure_configured(input_resolutions);
            }
        };

        match self {
            BuiltinNode::Configured(ref configured) => configured,
            BuiltinNode::Constructed(_) => unreachable!("Should be configured by previous call"),
        }
    }
}
