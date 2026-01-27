use wgpu::{Device, TextureView};
use winit::dpi::Size;

use crate::{render::{entity_meshs::MeshInstanceRaw, init::sun_shadows::InitSunShadow, wgpu::{RenderState, SunShadow}}, utils::Vertex};

pub struct InitGbufferInfo {
    pub base_color_gbuffer_view: TextureView,
    pub lighting_gbuffer_view: TextureView,
    pub base_color_gbuffer_sampler: wgpu::Sampler,
    pub lighting_gbuffer_sampler: wgpu::Sampler,
    pub gbuffers_bind_group: wgpu::BindGroup,
    pub gbuffers_bind_group_layout: wgpu::BindGroupLayout,
    pub material_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_view: TextureView,
    pub material_gbuffer_view: TextureView,
    pub gbuffer_render_pipeline : wgpu::RenderPipeline,
}

impl InitGbufferInfo {
    pub async fn new(device: &wgpu::Device, size : &winit::dpi::PhysicalSize<u32>, depth_view: &wgpu::TextureView, depth_sampler: &wgpu::Sampler, camera_bind_group_layout: &wgpu::BindGroupLayout, sun_shadow : &InitSunShadow) -> InitGbufferInfo {
        let full_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/full_shader.wgsl").into()),
        });
        
        //create texture infos
        let base_color_gbuffer = create_base_color_gbuffer(&device, size.width, size.height);
        let lighting_gbuffer = create_lighting_gbuffer(&device, size.width, size.height);
        let normal_gbuffer = create_normal_gbuffer(&device, size.width, size.height);
        let material_gbuffer = create_normal_gbuffer(&device, size.width, size.height);
        
        //make views
        let base_color_gbuffer_view = base_color_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
        let lighting_gbuffer_view = lighting_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_gbuffer_view = normal_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
        let material_gbuffer_view = material_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
        
        //make samplers
        let base_color_gbuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let lighting_gbuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let normal_gbuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let material_gbuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        
        //bind group layout
        let gbuffers_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                //base color
                wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            //lighting
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            //normal
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            //material
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            //depth
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth, 
                    view_dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 9,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            }
            ],
            label: Some("Gbuffers Bind Group Layout"),
        });
        
        //bind group
        //remember to also update in resize
        let gbuffers_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gbuffers_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&base_color_gbuffer_view)},
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&base_color_gbuffer_sampler)},
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&lighting_gbuffer_view)},
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&lighting_gbuffer_sampler)},
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&normal_gbuffer_view)},
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&normal_gbuffer_sampler)},
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&material_gbuffer_view)},
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(&material_gbuffer_sampler)},
                wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(&depth_view)},
                wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::Sampler(&depth_sampler)},
            ],
            label: Some("Gbuffers Bind Group"),
        });
            
        let create_gbuffers_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &sun_shadow.bind_group_layout
            ],
            push_constant_ranges: &[],
        });
                
                //full render pipeline
        let gbuffer_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gbuffer Render Pipeline"),
            layout: Some(&create_gbuffers_pipeline_layout),
            vertex: wgpu::VertexState {
            module: &full_shader,
            entry_point: Some("vs_main"),
            buffers: &[
                Vertex::desc(),
                MeshInstanceRaw::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &full_shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
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
        
        InitGbufferInfo {
            base_color_gbuffer_view,
            lighting_gbuffer_view,
            base_color_gbuffer_sampler,
            lighting_gbuffer_sampler,
            gbuffers_bind_group,
            gbuffers_bind_group_layout,
            normal_gbuffer_view,
            material_gbuffer_view,
            normal_gbuffer_sampler,
            material_gbuffer_sampler,
            gbuffer_render_pipeline,
        }
    }
}
    
    


pub fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
             | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Depth32Float],
    })
}

pub fn create_base_color_gbuffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Base Color Gbuffer"),
        size: size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    })
}

pub fn create_lighting_gbuffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Lighting Gbuffer"),
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

pub fn create_material_gbuffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Material Gbuffer"),
        size: size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm, //
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    })
}

pub fn create_normal_gbuffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Normal Gbuffer"),
        size: size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    })
}

pub fn update_render_state_gbuffer(render_state : &mut RenderState) {
    let depth_texture = create_depth_texture(&render_state.device,render_state.config.width,render_state.config.height);
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    render_state.depth_texture = depth_texture;
    render_state.depth_view = depth_view;
    render_state.is_surface_configured = true;

    //update gbuffers
    let base_color_gbuffer = create_base_color_gbuffer(&render_state.device, render_state.config.width, render_state.config.height);
    let lighting_gbuffer = create_lighting_gbuffer(&render_state.device, render_state.config.width, render_state.config.height);
    let normal_gbuffer = create_normal_gbuffer(&render_state.device, render_state.config.width, render_state.config.height);
    let material_gbuffer = create_material_gbuffer(&render_state.device, render_state.config.width, render_state.config.height);

    //make views
    render_state.base_color_gbuffer_view = base_color_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
    render_state.lighting_gbuffer_view = lighting_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
    render_state.normal_gbuffer_view = normal_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
    render_state.material_gbuffer_view = material_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());

    //remember to also update in init
    render_state.gbuffers_bind_group = render_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &render_state.gbuffers_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&render_state.base_color_gbuffer_view)},
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&render_state.base_color_gbuffer_sampler)},
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&render_state.lighting_gbuffer_view)},
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&render_state.lighting_gbuffer_sampler)},
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&render_state.normal_gbuffer_view)},
            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&render_state.normal_gbuffer_sampler)},
            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&render_state.material_gbuffer_view)},
            wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(&render_state.material_gbuffer_sampler)},
            wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(&render_state.depth_view)},
            wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::Sampler(&render_state.depth_sampler)},
        ],
        label: Some("Gbuffers Bind Group"),
    });
}