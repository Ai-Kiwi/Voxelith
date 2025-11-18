use std::{collections::HashMap, sync::Arc, time::Instant, vec};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::{Texture, TextureView, util::DeviceExt};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{render::{LEVEL_1_LOD_DISTANCE, LEVEL_2_LOD_DISTANCE, LEVEL_3_LOD_DISTANCE, LEVEL_4_LOD_DISTANCE, RenderData, RenderThreadChannels, camera::{Camera, CameraUniform}, init_frame_render}, utils::{Vec2, Vec3, Vertex}};

pub fn get_distance_to_camera_unsquared(render_state : &RenderState, x : f32, y : f32, z : f32) -> f32 {
    let dx = render_state.camera.eye.x - x;
    let dy = render_state.camera.eye.y - y;
    let dz = render_state.camera.eye.z - z;

    return dx*dx + dy*dy + dz*dz;
}


fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
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

fn chunk_to_batch(position_x : f32,position_y : f32,position_z : f32, batch_size : f32) -> (i32,i32,i32) {
    return (
        position_x.div_euclid(16.0 * batch_size) as i32,
        position_y.div_euclid(16.0 * batch_size) as i32, 
        position_z.div_euclid(16.0 * batch_size) as i32
    );
}


struct RenderStats {
    draw_calls: u32,
    triangles: u32,
}

impl RenderStats {
    fn new() -> Self {
        Self { draw_calls: 0, triangles: 0 }
    }
}

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub window: Arc<Window>,
    pub data: RenderData,
    pub render_channels : RenderThreadChannels,
    pub depth_texture: Texture,
    pub depth_view: TextureView,

    pub last_frame_time : Instant,
    pub delta_time : f32,

    pub keys_down : HashMap<KeyCode,()>,
    pub keys_pressed : HashMap<KeyCode,()>,
    pub keys_released : HashMap<KeyCode,()>,

    pub old_mouse_position: Option<winit::dpi::PhysicalPosition<f64>>,
    pub mouse_position_delta: Vec2,
}

impl<'a> RenderState {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>, render_channels : RenderThreadChannels) -> anyhow::Result<RenderState> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

            let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
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

        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 10000.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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

        let depth_texture = create_depth_texture(&device,size.width,size.height);
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            data : RenderData::new(),
            window,
            render_channels: render_channels,
            keys_down: HashMap::new(),
            keys_pressed: HashMap::new(),
            keys_released: HashMap::new(),
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            old_mouse_position: None,
            mouse_position_delta: Vec2::new(0.0, 0.0),
            depth_texture,
            depth_view,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            let depth_texture = create_depth_texture(&self.device,width,height);
            let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;


            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
            self.is_surface_configured = true;
        }    
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        #[cfg(feature = "perf_logs")]
        let full_screen_draw_start_time = Instant::now();
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }
                
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Far Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                view: &self.depth_view, 
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }), 
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        render_pass.set_pipeline(&self.render_pipeline);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        
        #[cfg(feature = "perf_logs")]
        let mut stats = RenderStats::new();

        let camera_direction_normal = Vec3::new(
            self.camera.target.x - self.camera.eye.x, 
            self.camera.target.y - self.camera.eye.y, 
            self.camera.target.z - self.camera.eye.z
        ).normalize();


        let camera_position = Vec3::new(
            self.camera.eye.x,
            self.camera.eye.y,
            self.camera.eye.z
        );

        #[cfg(feature = "perf_logs")]
        let setup_start = Instant::now();

        let mut lvl0_chunk_render_list: Vec<(i32,i32,i32)> = Vec::new();
        let lvl0_chunk_range = (LEVEL_1_LOD_DISTANCE / 16.0) as i32;
        let lvl0_middle_chunk_place = chunk_to_batch(camera_position.x, camera_position.y, camera_position.z, 1.0);
        
        for x in ((lvl0_middle_chunk_place.0 as i32)-lvl0_chunk_range)..((lvl0_middle_chunk_place.0 as i32)+lvl0_chunk_range) {
            for y in ((lvl0_middle_chunk_place.1 as i32)-lvl0_chunk_range)..((lvl0_middle_chunk_place.1 as i32)+lvl0_chunk_range) {
                for z in ((lvl0_middle_chunk_place.2 as i32)-lvl0_chunk_range)..((lvl0_middle_chunk_place.2 as i32)+lvl0_chunk_range) {
                    let camera_chunk_normal = Vec3::new(
                        (x * 16 + 8) as f32 - self.camera.eye.x,
                        (y * 16 + 8) as f32 - self.camera.eye.y, 
                        (z * 16 + 8) as f32 - self.camera.eye.z
                    ).normalize();
                
                    let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                    if cos_angle < 0.707 { //around 45 degrees
                        continue;
                    }
                    lvl0_chunk_render_list.push((x,y,z));
                }
            }
        }

        let mut chunk_batch_render_list: Vec<(i32,i32,i32,u8)> = Vec::new();
        let lvl1_chunk_range = (LEVEL_2_LOD_DISTANCE / 32.0) as i32;
        let lvl1_middle_chunk_place = chunk_to_batch(camera_position.x, camera_position.y, camera_position.z, 2.0);
        
        for x in ((lvl1_middle_chunk_place.0 as i32)-lvl1_chunk_range)..((lvl1_middle_chunk_place.0 as i32)+lvl1_chunk_range) {
            for y in ((lvl1_middle_chunk_place.1 as i32)-lvl1_chunk_range)..((lvl1_middle_chunk_place.1 as i32)+lvl1_chunk_range) {
                for z in ((lvl1_middle_chunk_place.2 as i32)-lvl1_chunk_range)..((lvl1_middle_chunk_place.2 as i32)+lvl1_chunk_range) {
                    //tests if it is inside the old chunk place
                    if (x-lvl1_middle_chunk_place.0).abs() < lvl0_chunk_range / 2 && (y-lvl1_middle_chunk_place.1).abs() < lvl0_chunk_range / 2 && (z-lvl1_middle_chunk_place.2).abs() < lvl0_chunk_range / 2 {
                        continue;
                    }

                    let camera_chunk_normal = Vec3::new(
                        (x * 32 + 16) as f32 - self.camera.eye.x,
                        (y * 32 + 16) as f32 - self.camera.eye.y, 
                        (z * 32 + 16) as f32 - self.camera.eye.z
                    ).normalize();
                
                    let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                    if cos_angle < 0.707 { //around 45 degrees
                        continue;
                    }

                    chunk_batch_render_list.push((x,y,z,1));
                }
            }
        }

        let lvl2_chunk_range = (LEVEL_3_LOD_DISTANCE / 64.0) as i32;
        let lvl2_middle_chunk_place = chunk_to_batch(camera_position.x, camera_position.y, camera_position.z, 4.0);
        
        for x in ((lvl2_middle_chunk_place.0 as i32)-lvl2_chunk_range)..((lvl2_middle_chunk_place.0 as i32)+lvl2_chunk_range) {
            for y in ((lvl2_middle_chunk_place.1 as i32)-lvl2_chunk_range)..((lvl2_middle_chunk_place.1 as i32)+lvl2_chunk_range) {
                for z in ((lvl2_middle_chunk_place.2 as i32)-lvl2_chunk_range)..((lvl2_middle_chunk_place.2 as i32)+lvl2_chunk_range) {
                    //tests if it is inside the old chunk place
                    if (x-lvl2_middle_chunk_place.0).abs() < lvl1_chunk_range / 2 && (y-lvl2_middle_chunk_place.1).abs() < lvl1_chunk_range / 2 && (z-lvl2_middle_chunk_place.2).abs() < lvl1_chunk_range / 2 {
                        continue;
                    }

                    let camera_chunk_normal = Vec3::new(
                        (x * 64 + 32) as f32 - self.camera.eye.x,
                        (y * 64 + 32) as f32 - self.camera.eye.y, 
                        (z * 64 + 23) as f32 - self.camera.eye.z
                    ).normalize();
                
                    let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                    if cos_angle < 0.707 { //around 45 degrees
                        continue;
                    }
                    
                    chunk_batch_render_list.push((x,y,z,2));
                }
            }
        }

        let lvl3_chunk_range = (LEVEL_4_LOD_DISTANCE / 128.0) as i32;
        let lvl3_middle_chunk_place = chunk_to_batch(camera_position.x, camera_position.y, camera_position.z, 8.0);
        
        for x in ((lvl3_middle_chunk_place.0 as i32)-lvl3_chunk_range)..((lvl3_middle_chunk_place.0 as i32)+lvl3_chunk_range) {
            for y in ((lvl3_middle_chunk_place.1 as i32)-lvl3_chunk_range)..((lvl3_middle_chunk_place.1 as i32)+lvl3_chunk_range) {
                for z in ((lvl3_middle_chunk_place.2 as i32)-lvl3_chunk_range)..((lvl3_middle_chunk_place.2 as i32)+lvl3_chunk_range) {
                    //tests if it is inside the old chunk place
                    if (x-lvl3_middle_chunk_place.0).abs() < lvl2_chunk_range / 2 && (y-lvl3_middle_chunk_place.1).abs() < lvl2_chunk_range / 2 && (z-lvl3_middle_chunk_place.2).abs() < lvl2_chunk_range / 2 {
                        continue;
                    }

                    let camera_chunk_normal = Vec3::new(
                        (x * 128 + 64) as f32 - self.camera.eye.x,
                        (y * 128 + 64) as f32 - self.camera.eye.y, 
                        (z * 128 + 64) as f32 - self.camera.eye.z
                    ).normalize();
                
                    let cos_angle = camera_chunk_normal.dot(&camera_direction_normal);
                    if cos_angle < 0.707 { //around 45 degrees
                        continue;
                    }
                    
                    chunk_batch_render_list.push((x,y,z,4));
                }
            }
        }


        let final_render_batch_mesh_list : Vec<_> = chunk_batch_render_list
        .par_iter()
        .filter_map(|&(x, y, z, batch_level)| {
            self.data.chunk_batch_meshs
            .get(&(x, y, z, batch_level))
            .filter(|mesh| mesh.num_indices > 0)
        })
        .collect();

        let final_render_mesh_list : Vec<_> = lvl0_chunk_render_list
        .par_iter()
        .filter_map(|&(x, y, z)| {
            self.data.chunk_meshs
            .get(&(x, y, z))
            .filter(|mesh| mesh.main_lod.num_indices > 0)
        })
        .collect();


        #[cfg(feature = "perf_logs")]
        println!("Started setup {}ms",setup_start.elapsed().as_millis());
        #[cfg(feature = "perf_logs")]
        let render_start = Instant::now();


        for mesh in final_render_mesh_list {
            render_pass.set_vertex_buffer(0, mesh.main_lod.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.main_lod.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.main_lod.num_indices, 0, 0..1);

            #[cfg(feature = "perf_logs")]
            {
                stats.draw_calls += 1;
                stats.triangles += mesh.main_lod.num_indices / 3;
            }
        };

        for mesh in final_render_batch_mesh_list {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);

            #[cfg(feature = "perf_logs")]
            {
                stats.draw_calls += 1;
                stats.triangles += mesh.num_indices / 3;
            }
        };
        drop(render_pass);

        #[cfg(feature = "perf_logs")]
        println!("renderer game {}ms",render_start.elapsed().as_millis());
 

        #[cfg(feature = "perf_logs")]
        println!(
            "Draw calls: {}, Triangles: {}",
            stats.draw_calls,
            stats.triangles
        );

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        #[cfg(feature = "perf_logs")]
        println!("full screen draw {}ms",full_screen_draw_start_time.elapsed().as_millis());

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {
                if is_pressed {
                    if !self.keys_down.contains_key(&code) {
                        self.keys_down.insert(code, ());
                        self.keys_pressed.insert(code, ());
                    }
                }else{
                    self.keys_down.remove(&code);
                    self.keys_released.insert(code, ());
                }
            }
        }
    }

    pub fn update(&mut self) {
        init_frame_render(self);
    }

}

