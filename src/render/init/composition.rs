use wgpu::{Device, PipelineLayout};

use crate::render::init::gbuffer::InitGbufferInfo;

pub struct InitCompositionInfo {
    pub composition_pipeline_layout: PipelineLayout,
    pub composition_render_pipeline : wgpu::RenderPipeline,
}

impl InitCompositionInfo {
    pub fn new(device : &Device, gbuffer_info : &InitGbufferInfo, camera_bind_group_layout: &wgpu::BindGroupLayout,) -> InitCompositionInfo {
        let composition_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/composition_shader.wgsl").into()),
        });
        
        let composition_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Composition Pipeline Layout"),
            bind_group_layouts: &[
                &gbuffer_info.gbuffers_bind_group_layout,
                &camera_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let composition_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Composition Render Pipeline"),
            layout: Some(&composition_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &composition_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &composition_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        #[cfg(target_os = "linux")] format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        #[cfg(target_os = "windows")] format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
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
                depth_compare: wgpu::CompareFunction::Always,
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

        InitCompositionInfo {
            composition_pipeline_layout,
            composition_render_pipeline,
        }
    }
}