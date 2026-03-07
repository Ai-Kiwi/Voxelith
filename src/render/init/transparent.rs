use wgpu::{Device, PipelineLayout};

use crate::{render::{entity_meshs::MeshInstanceRaw, init::gbuffer::InitGbufferInfo}, utils::Vertex};

pub struct InitTransparentInfo {
    pub transparent_pipeline_layout: PipelineLayout,
    pub transparent_render_pipeline : wgpu::RenderPipeline,
}

impl InitTransparentInfo {
    pub fn new(device : &Device, gbuffer_info : &InitGbufferInfo, camera_bind_group_layout: &wgpu::BindGroupLayout, surface_format : wgpu::TextureFormat) -> InitTransparentInfo {
        let transparent_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Transparent Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/transparent_shader.wgsl").into()),
        });

        let transparent_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Transparent Pipeline Layout"),
            bind_group_layouts: &[
                &gbuffer_info.gbuffers_bind_group_layout,
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let transparent_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Transparent Render Pipeline"),
            layout: Some(&transparent_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &transparent_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                    MeshInstanceRaw::desc()
                ],                
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &transparent_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        InitTransparentInfo {
            transparent_pipeline_layout,
            transparent_render_pipeline,
        }
    }
}