use std::{collections::HashMap, sync::Arc, time::Instant};

use cgmath::Point3;
use egui_wgpu::Renderer;
use wgpu::{BindGroupLayout, Buffer, Device, Texture, TextureView, util::DeviceExt};
use winit::{keyboard::KeyCode, window::Window};

use crate::{render::{RenderFrameThreadPerformanceInfo, camera::{CameraUniform, OrthographicCamera, PerspectiveCamera}, mesh::MeshBuffer, render_frame::gui::GuiInfo}, utils::Vec2};

pub fn get_distance_to_camera_unsquared(camera : &PerspectiveCamera, x : f32, y : f32, z : f32) -> f32 {
    let dx = camera.position.x - x;
    let dy = camera.position.y - y;
    let dz = camera.position.z - z;

    return dx*dx + dy*dy + dz*dz;
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
pub struct RenderState {
    //gpu related data
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub basic_mesh_render_pipeline: wgpu::RenderPipeline,
    pub gbuffer_render_pipeline : wgpu::RenderPipeline,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub window: Arc<Window>,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    
    //other stuff that is just helped for engine itself
    pub last_frame_time : Instant,
    pub start_time : Instant,
    pub delta_time : f32,
    pub keys_down : HashMap<KeyCode,()>,
    pub keys_pressed : HashMap<KeyCode,()>,
    pub keys_released : HashMap<KeyCode,()>,
    pub mouse_position_delta: Vec2,
    pub mouse_position: Vec2,
    //buffers used, although store game data they are overridden and temporary.
    pub temporary_move_buffer : Buffer, 
    pub mesh_id_upto : u64,
    pub mesh_buffers : Vec<MeshBuffer>,


    //gui related stuff. Also engine
    pub egui_renderer : Renderer,
    pub egui_context : egui::Context,
    pub egui_winit : egui_winit::State,

    //window state
    pub game_selected : bool,
    pub fullscreen : bool,
    pub gui_info : GuiInfo,

    pub performance_info : RenderFrameThreadPerformanceInfo,

    pub base_color_gbuffer_view: TextureView,
    pub lighting_gbuffer_view: TextureView,
    pub base_color_gbuffer_sampler: wgpu::Sampler,
    pub lighting_gbuffer_sampler: wgpu::Sampler,
    pub gbuffers_bind_group: wgpu::BindGroup,
    pub gbuffers_bind_group_layout: wgpu::BindGroupLayout,
    pub composition_pipeline_layout: wgpu::PipelineLayout,
    pub composition_render_pipeline: wgpu::RenderPipeline,
    pub material_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_sampler: wgpu::Sampler,
    pub normal_gbuffer_view: TextureView,
    pub material_gbuffer_view: TextureView,

    pub sun_shadow_lod_0 : SunShadow,
    pub sun_shadow_lod_1 : SunShadow,
    pub sun_shadow_lod_2 : SunShadow,
    pub sun_shadow_lod_3 : SunShadow,
    pub sun_shadow_render_pipeline: wgpu::RenderPipeline,
    pub sun_shadow_textures_bind_group: wgpu::BindGroup,
}

pub struct SunShadow {
    pub camera : OrthographicCamera,
    pub camera_uniform : CameraUniform,
    pub camera_buffer: Buffer,
    pub texture_view: TextureView,
    pub texture_sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl SunShadow {
    pub fn new(device: &Device, lod_level : u32, layout : &BindGroupLayout) -> SunShadow {
        let mut sun_shadows_lod_camera = OrthographicCamera::new();
        sun_shadows_lod_camera.width = 128.0 * (lod_level as f32);
        sun_shadows_lod_camera.height = 128.0 * (lod_level as f32);
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
            size: wgpu::Extent3d { width : 1024 / lod_level, height : 1024 / lod_level, depth_or_array_layers : 1 },
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



impl<'a> RenderState {

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            let depth_texture = create_depth_texture(&self.device,width,height);
            let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
            self.is_surface_configured = true;

            //update gbuffers

            //create texture infos
            let base_color_gbuffer = create_base_color_gbuffer(&self.device, width, height);
            let lighting_gbuffer = create_lighting_gbuffer(&self.device, width, height);
            let normal_gbuffer = create_normal_gbuffer(&self.device, width, height);
            let material_gbuffer = create_material_gbuffer(&self.device, width, height);

            //make views
            self.base_color_gbuffer_view = base_color_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
            self.lighting_gbuffer_view = lighting_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
            self.normal_gbuffer_view = normal_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());
            self.material_gbuffer_view = material_gbuffer.create_view(&wgpu::TextureViewDescriptor::default());

            //make samplers
            self.base_color_gbuffer_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            self.lighting_gbuffer_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            self.normal_gbuffer_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            self.material_gbuffer_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());

            //remember to also update in init
            self.gbuffers_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.gbuffers_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&self.base_color_gbuffer_view)},
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.base_color_gbuffer_sampler)},
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.lighting_gbuffer_view)},
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.lighting_gbuffer_sampler)},
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&self.normal_gbuffer_view)},
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&self.normal_gbuffer_sampler)},
                    wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&self.material_gbuffer_view)},
                    wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(&self.material_gbuffer_sampler)},
                ],
                label: Some("Gbuffers Bind Group"),
            });
        }    
    }

}

