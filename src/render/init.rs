use std::{collections::HashMap, sync::Arc, time::Instant};

use egui::ViewportId;
use egui_wgpu::{Renderer, RendererOptions};
use wgpu::{ExperimentalFeatures, util::DeviceExt};
use winit::window::{Theme, Window};

use crate::{render::{RenderFrameThreadPerformanceInfo, camera::{CameraUniform, OrthographicCamera, PerspectiveCamera}, render_frame::gui::GuiInfo, wgpu::{RenderState, SunShadow, create_base_color_gbuffer, create_depth_texture, create_lighting_gbuffer, create_material_gbuffer, create_normal_gbuffer}}, render_game::MAP_VRAM_SIZE, utils::{Vec2, Vertex}};

impl RenderState {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>) -> anyhow::Result<RenderState> {
        let size: winit::dpi::PhysicalSize<u32> = window.inner_size();


        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        //setup gpu
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        
        if !adapter.features().contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT) {
            panic!("multi indirect draw not supported on gpu")
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
                required_limits: wgpu::Limits {
                    max_buffer_size: MAP_VRAM_SIZE,
                    ..Default::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
                experimental_features: ExperimentalFeatures::default(),
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let basic_mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("basic_mesh_shader.wgsl").into()),
        });

        let full_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("full_shader.wgsl").into()),
        });

        let composition_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("composition_shader.wgsl").into()),
        });

        let light_depth_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("light_depth.wgsl").into()),
        });

        let temporary_move_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Temporary Move Buffer"),
            size: 2 * 1024 * 1024, // 1 MB
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut camera = PerspectiveCamera::new();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj_prespec(&mut camera, config.width, config.height);



        //setup needed buffers and info for them
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );


        //sun shadow
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

        let sun_shadow_lod_0 = SunShadow::new(&device, 1, &sun_shadow_bind_group_layout);
        let sun_shadow_lod_1 = SunShadow::new(&device, 2, &sun_shadow_bind_group_layout);
        let sun_shadow_lod_2 = SunShadow::new(&device, 4, &sun_shadow_bind_group_layout);
        let sun_shadow_lod_3 = SunShadow::new(&device, 8, &sun_shadow_bind_group_layout);

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

        let sun_shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                    Vertex::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
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

        //camera bind group
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        //create texture infos
        let base_color_gbuffer = create_base_color_gbuffer(&device, size.width, size.height);
        let lighting_gbuffer = create_lighting_gbuffer(&device, size.width, size.height);
        let normal_gbuffer = create_normal_gbuffer(&device, size.width, size.height);
        let material_gbuffer = create_material_gbuffer(&device, size.width, size.height);

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
            ],
            label: Some("Gbuffers Bind Group"),
        });

        let create_gbuffers_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &sun_shadow_textures_bind_group_layout
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
                    Vertex::desc()
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
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // closer fragments overwrite farther ones
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

        //composition render
        let composition_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Composition Pipeline Layout"),
                bind_group_layouts: &[
                    &gbuffers_bind_group_layout
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
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
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

        //basic mesh render pipeline
        let basic_mesh_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout
                ],
                push_constant_ranges: &[],
        });

        let basic_mesh_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Basic Mesh Render Pipeline"),
            layout: Some(&basic_mesh_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &basic_mesh_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &basic_mesh_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // closer fragments overwrite farther ones
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

        let depth_texture = create_depth_texture(&device,size.width,size.height);
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        //setup egui
        let egui_renderer = Renderer::new(&device, surface_format, RendererOptions { 
            msaa_samples: 1, 
            depth_stencil_format: None, 
            dithering: false, 
            predictable_texture_filtering: false 
        });

        let egui_context: egui::Context = egui::Context::default();
        let egui_winit: egui_winit::State = egui_winit::State::new(egui_context.clone(), ViewportId::ROOT, &window, Some(1.0), Some(Theme::Dark), Some(4096));

        let performance_info = RenderFrameThreadPerformanceInfo {
            total_tick_time: 0.0,
            total_render_time: 0.0,
            main_game_tick: 0.0,
            mesh_creator_tick: 0.0,
            update_mesh_buffer: 0.0,
            start_render_time: 0.0,
            main_content_render_time: 0.0,
            render_gui_time: 0.0,
            finish_render_time: 0.0,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            basic_mesh_render_pipeline,
            gbuffer_render_pipeline,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            window,
            keys_down: HashMap::new(),
            keys_pressed: HashMap::new(),
            keys_released: HashMap::new(),
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            mouse_position_delta: Vec2::new(0.0, 0.0),
            mouse_position: Vec2::new(0.0, 0.0),
            depth_texture,
            depth_view,
            egui_renderer,
            egui_context,
            egui_winit,
            start_time : Instant::now(),
            game_selected: true,
            fullscreen: false,
            gui_info: GuiInfo::new(),
            mesh_id_upto : 1, // start at 1 as 0 means empty data
            performance_info,
            mesh_buffers: Vec::new(),
            temporary_move_buffer,
            base_color_gbuffer_view,
            lighting_gbuffer_view,
            base_color_gbuffer_sampler,
            lighting_gbuffer_sampler,
            gbuffers_bind_group,
            gbuffers_bind_group_layout,
            composition_pipeline_layout,
            composition_render_pipeline,
            normal_gbuffer_view,
            material_gbuffer_view,
            normal_gbuffer_sampler,
            material_gbuffer_sampler,
            sun_shadow_lod_0,
            sun_shadow_lod_1,
            sun_shadow_lod_2,
            sun_shadow_lod_3,
            sun_shadow_render_pipeline,
            sun_shadow_textures_bind_group
        })
    }
}

//render buffers
//base surface color
//depth/position
//material data
//light data
//sun data