use std::{collections::HashMap, sync::Arc, time::Instant};

use egui::ViewportId;
use egui_wgpu::{Renderer, RendererOptions};
use wgpu::{ExperimentalFeatures, util::DeviceExt};
use winit::window::{Theme, Window};

pub mod sun_shadows;
pub mod gbuffer;
pub mod composition;
pub mod entity_meshs;

use crate::{render::{MAP_VRAM_SIZE, RenderFrameThreadPerformanceInfo, camera::{CameraUniform, PerspectiveCamera}, init::{composition::InitCompositionInfo, entity_meshs::InitEntityMeshs, gbuffer::{InitGbufferInfo, create_depth_texture}, sun_shadows::InitSunShadow}, render_frame::gui::GuiInfo, wgpu::RenderState}, utils::{Vec2, Vertex}};

pub async fn init_render_state(window: Arc<Window>) -> anyhow::Result<RenderState>  {
    let size: winit::dpi::PhysicalSize<u32> = window.inner_size();

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    //setup gpu
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await?;
    
    if !adapter.features().contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT) {
        panic!("multi indirect draw not supported on gpu")
    }

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
        required_limits: wgpu::Limits {
            max_buffer_size: MAP_VRAM_SIZE,
            ..Default::default()
        },
        memory_hints: Default::default(),
        trace: wgpu::Trace::Off,
        experimental_features: ExperimentalFeatures::default(),
    }).await?;

    let depth_texture = create_depth_texture(&device,size.width,size.height);
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    let surface_caps = surface.get_capabilities(&adapter);
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

    let temporary_move_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Temporary Move Buffer"),
        size: 2 * 1024 * 1024, // 2 MB
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let mut camera = PerspectiveCamera::new();
    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj_prespec(&mut camera, config.width, config.height);
    let camera_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );
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
        label: Some("camera bind group layout"),
    });
    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }
        ],
        label: Some("camera bind group"),
    });

    //sun shadow
    let sun_shadow = InitSunShadow::new(&device).await;

    //create gbuffer 
    let gbuffer_info = InitGbufferInfo::new(&device, &size, &depth_view, &depth_sampler, &camera_bind_group_layout, &sun_shadow).await;

    //composition render
    let composition = InitCompositionInfo::new(&device, &gbuffer_info, &camera_bind_group_layout);

    //entity meshs
    let entity_mesh_data = InitEntityMeshs::new(&device,&queue);

    //setup egui
    let egui_renderer = Renderer::new(&device, surface_format, RendererOptions { 
        msaa_samples: 1, 
        depth_stencil_format: None, 
        dithering: false, 
        predictable_texture_filtering: false 
    });

    let egui_context: egui::Context = egui::Context::default();
    let egui_winit: egui_winit::State = egui_winit::State::new(egui_context.clone(), ViewportId::ROOT, &window, Some(1.0), Some(Theme::Dark), Some(4096));

    //performance monitoring info
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

    Ok(RenderState {
        surface,
        device,
        queue,
        config,
        is_surface_configured: false,
        gbuffer_render_pipeline : gbuffer_info.gbuffer_render_pipeline,
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
        depth_sampler,
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
        base_color_gbuffer_view : gbuffer_info.base_color_gbuffer_view,
        lighting_gbuffer_view : gbuffer_info.lighting_gbuffer_view,
        base_color_gbuffer_sampler : gbuffer_info.base_color_gbuffer_sampler,
        lighting_gbuffer_sampler : gbuffer_info.lighting_gbuffer_sampler,
        gbuffers_bind_group : gbuffer_info.gbuffers_bind_group,
        gbuffers_bind_group_layout : gbuffer_info.gbuffers_bind_group_layout,
        composition_pipeline_layout: composition.composition_pipeline_layout,
        composition_render_pipeline: composition.composition_render_pipeline,
        normal_gbuffer_view : gbuffer_info.normal_gbuffer_view,
        material_gbuffer_view : gbuffer_info.material_gbuffer_view,
        normal_gbuffer_sampler : gbuffer_info.normal_gbuffer_sampler,
        material_gbuffer_sampler : gbuffer_info.material_gbuffer_sampler,
        sun_shadow_lod_0 : sun_shadow.sun_shadow_lod_0,
        sun_shadow_lod_1 : sun_shadow.sun_shadow_lod_1,
        sun_shadow_lod_2 : sun_shadow.sun_shadow_lod_2,
        sun_shadow_lod_3 : sun_shadow.sun_shadow_lod_3,
        sun_shadow_render_pipeline : sun_shadow.pipeline,
        sun_shadow_textures_bind_group : sun_shadow.bind_group,
        mesh_id_reference: entity_mesh_data.mesh_id_reference,
        mesh_instances: entity_mesh_data.instances,
        entity_meshs_buffer: entity_mesh_data.meshs_buffer,
        blank_instance_info : entity_mesh_data.blank_instance_info,
    })
}