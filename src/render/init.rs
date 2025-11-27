use std::{collections::HashMap, sync::Arc, time::Instant};

use dashmap::DashMap;
use egui::ViewportId;
use egui_wgpu::{Renderer, RendererOptions};
use rayon::vec;
use wgpu::{ExperimentalFeatures, RenderPass, util::DeviceExt};
use winit::window::{Theme, Window};

use crate::{render::{RenderFrameThreadPerformanceInfo, camera::{Camera, CameraUniform}, mesh::FreeBufferSpace, render_frame::gui::GuiInfo, wgpu::{RenderState, create_depth_texture}}, render_game::MAP_VRAM_SIZE, utils::{Vec2, Vertex}};

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut camera = Camera::new();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&mut camera, config.width, config.height);

        //setup needed buffers and info for them
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let mesh_buffer = device.create_buffer( &wgpu::BufferDescriptor {
            label: Some("Multi chunk buffer"),
            usage: wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_SRC,
            size: MAP_VRAM_SIZE,
            mapped_at_creation: false, 
        });
        let mut free_mesh_buffer_ranges = Vec::new();
        free_mesh_buffer_ranges.push(FreeBufferSpace {
            byte_start: 0,
            byte_len: (MAP_VRAM_SIZE as u32) - 1,
        });

        let opaque_indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Buffer"),
            size: 16 * 1024 * 1024, // 16 MB
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transparent_indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Buffer"),
            size: 16 * 1024 * 1024, // 16 MB
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let temporary_move_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Temporary Move Buffer"),
            size: 2 * 1024 * 1024, // 1 MB
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let opaque_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Opaque Count Buffer"),
            contents: bytemuck::cast_slice(&[0 as u32]),
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        });

        let transparent_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transparent Count Buffer"),
            contents: bytemuck::cast_slice(&[0 as u32]),
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        });

        //setup render info
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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


        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout
                ],
                push_constant_ranges: &[],
        });

        let opaque_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Opaque Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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

        let transparent_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Transparent Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
                depth_compare: wgpu::CompareFunction::LessEqual, // closer fragments overwrite farther ones
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
        let mut egui_renderer = Renderer::new(&device, surface_format, RendererOptions { 
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
            percent_mesh_buffer_use: 0.0,
            percent_mesh_buffer_usable: 0.0,
            fragments_mesh_buffer: 0,
            bad_fragments_mesh_buffer: 0,
            buffer_defragmentation: false,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            opaque_render_pipeline,
            transparent_render_pipeline,
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
            opaque_indirect_buffer,
            transparent_indirect_buffer,
            temporary_move_buffer,
            opaque_count_buffer,
            transparent_count_buffer,
            depth_texture,
            depth_view,
            egui_renderer,
            egui_context,
            egui_winit,
            start_time : Instant::now(),
            game_selected: true,
            fullscreen: false,
            gui_info: GuiInfo::new(),
            mesh_buffer,
            free_mesh_buffer_ranges,
            meshs: DashMap::new(),
            mesh_id_upto : 1, // start at 1 as 0 means empty data
            performance_info,
        })
    }
}