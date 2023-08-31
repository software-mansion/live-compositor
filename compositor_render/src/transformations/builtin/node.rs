// use std::{collections::HashMap, sync::Arc, time::Duration};

// use compositor_common::scene::{
//     builtin_transformations::BuiltinTransformationSpec, NodeId, Resolution, ShaderParam,
// };
// use wgpu::util::DeviceExt;

// use crate::{
//     renderer::{texture::NodeTexture, WgpuCtx},
//     transformations::shader::{Shader},
// };

// use super::transformations::BuiltinTransformations;

// struct ShaderParamsBufferState {
//     bind_group: wgpu::BindGroup,
//     buffer: wgpu::Buffer,
//     params: ShaderParam,
// }

// pub struct BuiltinTransformationNode {
//     shader: Arc<Shader>,
//     shader_params_state: ShaderParamsBufferState,
//     spec: BuiltinTransformationSpec,
//     resolution: Resolution,
//     sources_resolutions: Vec<Option<Resolution>>,
// }

// impl BuiltinTransformationNode {
//     pub fn new(
//         ctx: &WgpuCtx,
//         shader: Arc<Shader>,
//         spec: BuiltinTransformationSpec,
//         resolution: Resolution,
//     ) -> Self {
//         Self {
//             shader,
//             shader_params_state: ShaderParamsBufferState::Empty,
//             spec,
//             resolution,
//             sources_resolutions: Vec::new(),
//         }
//     }

//     pub fn render(
//         &self,
//         sources: &[(&NodeId, &NodeTexture)],
//         target: &mut NodeTexture,
//         pts: Duration,
//     ) {
//         // TODO: temporary hack until builtins are stateless
//         if sources.len() == 1 && sources[0].1.is_empty() {
//             target.clear();
//             return;
//         }

//         let sources_resolutions: Vec<Option<Resolution>> = sources
//             .iter()
//             .map(|(_, &node_texture)| node_texture.resolution())
//             .collect();

//         if self.sources_resolutions != sources_resolutions {
//             self.update_params(sources_resolutions);
//             self.sources_resolutions = sources_resolutions;
//         }

//         let target = target.ensure_size(&self.shader.wgpu_ctx, self.resolution);
//         self.shader.render(
//             &self.shader_params_bind_group,
//             sources,
//             target,
//             pts,
//             self.clear_color,
//         )
//     }

//     fn update_params_buffer(&self, params: ShaderParam) {

//         let queue = self.shader.wgpu_ctx.queue;

//         match self.shader_params_state {
//             ShaderParamsBufferState::Created {
//                 bind_group, buffer, ..
//             } => {
//                 queue.write_buffer(&buffer, 0, &params_bytes);
//                 self.shader_params_state = ShaderParamsBufferState::Created {
//                     bind_group,
//                     buffer,
//                     params,
//                 };
//             }
//             ShaderParamsBufferState::Empty => {
//                 let shader_params_buffer =
//                     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                         label: Some("shader node custom params buffer"),
//                         usage: wgpu::BufferUsages::UNIFORM,
//                         contents: &params_bytes,
//                     });

//                 let shader_params_bind_group =
//                     device.create_bind_group(&wgpu::BindGroupDescriptor {
//                         label: Some("shader node params bind group"),
//                         layout: &self.shader.wgpu_ctx.shader_parameters_bind_group_layout,
//                         entries: &[
//                             wgpu::BindGroupEntry {
//                                 binding: 0,
//                                 resource: shader_params_buffer.as_entire_binding(),
//                             },
//                             wgpu::BindGroupEntry {
//                                 binding: 1,
//                                 resource: self
//                                     .shader
//                                     .wgpu_ctx
//                                     .compositor_provided_parameters_buffer
//                                     .as_entire_binding(),
//                             },
//                         ],
//                     });

//                 self.shader_params_state = ShaderParamsBufferState: {
//                     bind_group: shader_params_bind_group,
//                     buffer: shader_params_buffer,
//                     params,
//                 };
//             }
//         }
//     }

//     fn update_params(&self, sources_resolutions: Vec<Option<Resolution>>) {
//         let params =
//             BuiltinTransformations::params(&self.spec, sources_resolutions, self.resolution);
//         if let Some(shader_params) = params {
//             self.update_params_buffer(shader_params);
//         }
//     }
// }
