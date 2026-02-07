use wgpu::TextureView;

use crate::{render::{entity_meshs::MeshInstanceRaw, init::{gbuffer::InitGbufferInfo, sun_shadows::InitSunShadow}}, utils::Vertex};

pub struct InitVolumetricLightingInfo {
    pub volumetric_lighting_gbuffer_sampler: wgpu::Sampler,
    pub volumetric_lighting_gbuffer_view: TextureView,
    pub volumetric_lighting_render_pipeline : wgpu::RenderPipeline,
    pub volumetric_lighting_bind_group: wgpu::BindGroup,
    pub volumetric_lighting_bind_group_layout: wgpu::BindGroupLayout,
}

pub fn create_volumetric_lighting_gbuffer(device: &wgpu::Device, screen_width: u32, screen_height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: screen_width / 4,
        height : screen_height / 4,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Volumetric Lighting Gbuffer"),
        size: size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba16Float],
    })
}

impl InitVolumetricLightingInfo {
    pub fn new(device: &wgpu::Device, size : &winit::dpi::PhysicalSize<u32>, gbuffer_data: &InitGbufferInfo, camera_bind_group_layout: &wgpu::BindGroupLayout, sun_shadow : &InitSunShadow) -> InitVolumetricLightingInfo {
        let volumetric_lighting_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Volumetric Lighting Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/volumetric_lighting_shader.wgsl").into()),
        });
        
        //create texture infos
        let volumetric_lighting_gbuffer = create_volumetric_lighting_gbuffer(&device, size.width, size.height);
        
        //make views
        let volumetric_lighting_gbuffer_view = volumetric_lighting_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
        
        //make samplers
        let volumetric_lighting_gbuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        
        //bind group layout
        let volumetric_lighting_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Volumetric Lighting Bind Group Layout"),
        });
        
        //bind group
        //remember to also update in resize
        let volumetric_lighting_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &volumetric_lighting_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&volumetric_lighting_gbuffer_view)},
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&volumetric_lighting_gbuffer_sampler)},
            ],
            label: Some("Volumetric Lighting Bind Group"),
        });
            
        let create_gbuffers_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Volumetric Lighting Pipeline Layout"),
            bind_group_layouts: &[
                &gbuffer_data.gbuffers_bind_group_layout,
                &camera_bind_group_layout,
                &sun_shadow.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
                
        //volumetric lighting pipeline
        let volumetric_lighting_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Volumetric Lighting Render Pipeline"),
            layout: Some(&create_gbuffers_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &volumetric_lighting_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &volumetric_lighting_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        
        InitVolumetricLightingInfo {
            volumetric_lighting_gbuffer_sampler,
            volumetric_lighting_gbuffer_view,
            volumetric_lighting_render_pipeline,
            volumetric_lighting_bind_group_layout,
            volumetric_lighting_bind_group,
        }
    }
}