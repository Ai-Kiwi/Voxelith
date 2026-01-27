use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, PipelineLayout, TextureView, util::DeviceExt};

use crate::{render::{camera::{CameraUniform, OrthographicCamera}, entity_meshs::MeshInstanceRaw, wgpu::SunShadow}, utils::Vertex};

impl SunShadow {
    pub fn new(device: &Device, distance : f32, layout : &BindGroupLayout) -> SunShadow {
        let mut sun_shadows_lod_camera = OrthographicCamera::new();
        sun_shadows_lod_camera.width = distance;
        sun_shadows_lod_camera.height = distance;
        let mut sun_shadows_lod_camera_uniform = CameraUniform::new();
        sun_shadows_lod_camera_uniform.update_view_proj_ortho(&mut sun_shadows_lod_camera);
        let sun_shadows_lod_camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Shadow Camera Buffer"),
                contents: bytemuck::cast_slice(&[sun_shadows_lod_camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let sun_shadow_texture_lod = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Lod 0 Shadow Texture"),
            size: wgpu::Extent3d { width : 4096, height : 4096, depth_or_array_layers : 1 },   //ultra 8192, high 4096, medium 2048, low 1024, very low 512
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });
        let sun_shadow_texture_lod_view = sun_shadow_texture_lod.create_view(&wgpu::TextureViewDescriptor::default());
        let sun_shadow_texture_lod_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            compare: Some(wgpu::CompareFunction::Less),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let sun_shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sun_shadows_lod_camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("sun_shadow_bind_group"),
        });

        SunShadow {
            camera: sun_shadows_lod_camera,
            camera_uniform: sun_shadows_lod_camera_uniform,
            camera_buffer: sun_shadows_lod_camera_buffer,
            texture_view: sun_shadow_texture_lod_view,
            texture_sampler: sun_shadow_texture_lod_sampler,
            bind_group: sun_shadow_bind_group,
        }
    }
}

pub struct InitSunShadow {
    pub sun_shadow_lod_0 : SunShadow,
    pub sun_shadow_lod_1 : SunShadow,
    pub sun_shadow_lod_2 : SunShadow,
    pub sun_shadow_lod_3 : SunShadow,
    pub bind_group_layout : BindGroupLayout,
    pub bind_group : BindGroup,
    pub pipeline_layout : PipelineLayout,
    pub pipeline : wgpu::RenderPipeline
}

impl InitSunShadow {
    pub async fn new(device : &Device) -> InitSunShadow {
        let light_depth_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light_depth.wgsl").into()),
        });

        let sun_shadow_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("sun_shadow_bind_group_layout"),
        });

        let sun_shadow_lod_0 = SunShadow::new(&device, 128.0, &sun_shadow_bind_group_layout); //1
        let sun_shadow_lod_1 = SunShadow::new(&device, 384.0, &sun_shadow_bind_group_layout); //3
        let sun_shadow_lod_2 = SunShadow::new(&device, 1024.0, &sun_shadow_bind_group_layout); //8
        let sun_shadow_lod_3 = SunShadow::new(&device, 3072.0, &sun_shadow_bind_group_layout); //24

        let sun_shadow_textures_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                //lod0
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                //lod1
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                //lod2
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                //lod3
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Sun Shadow Textures Bind Group Layout"),
        });

        //bind group
        //remember to also update in resize
        let sun_shadow_textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sun_shadow_textures_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&sun_shadow_lod_0.texture_view)},
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sun_shadow_lod_0.texture_sampler)},
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Buffer(sun_shadow_lod_0.camera_buffer.as_entire_buffer_binding())},

                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&sun_shadow_lod_1.texture_view)},
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&sun_shadow_lod_1.texture_sampler)},
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Buffer(sun_shadow_lod_1.camera_buffer.as_entire_buffer_binding())},

                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&sun_shadow_lod_2.texture_view)},
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(&sun_shadow_lod_2.texture_sampler)},
                wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::Buffer(sun_shadow_lod_2.camera_buffer.as_entire_buffer_binding())},

                wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::TextureView(&sun_shadow_lod_3.texture_view)},
                wgpu::BindGroupEntry { binding: 10, resource: wgpu::BindingResource::Sampler(&sun_shadow_lod_3.texture_sampler)},
                wgpu::BindGroupEntry { binding: 11, resource: wgpu::BindingResource::Buffer(sun_shadow_lod_3.camera_buffer.as_entire_buffer_binding())},

            ],
            label: Some("Sun Shadow Textures Bind Group"),
        });

        let sun_shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Composition Pipeline Layout"),
                bind_group_layouts: &[
                    &sun_shadow_bind_group_layout
                ],
                push_constant_ranges: &[],
        });

        let sun_shadow_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sun Shadow Render Pipeline"),
            layout: Some(&sun_shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &light_depth_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                    MeshInstanceRaw::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
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

        Self {
            sun_shadow_lod_0: sun_shadow_lod_0,
            sun_shadow_lod_1: sun_shadow_lod_1,
            sun_shadow_lod_2: sun_shadow_lod_2,
            sun_shadow_lod_3: sun_shadow_lod_3,
            bind_group_layout: sun_shadow_textures_bind_group_layout,
            bind_group: sun_shadow_textures_bind_group,
            pipeline_layout: sun_shadow_pipeline_layout,
            pipeline: sun_shadow_render_pipeline,
        }
    }
}