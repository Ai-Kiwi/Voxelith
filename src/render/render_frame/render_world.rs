use std::collections::HashMap;

use wgpu::{CommandEncoder, wgt::DrawIndirectArgs};

use crate::render::{
    entity_meshs::{MeshId, MeshInstanceId},
    render_frame::{
        chunks::{create_chunk_draw_calls, render_opaque_chunks, render_transparent_chunks},
        entities::{create_entities_draw_calls, render_entities},
        shadows::render_sun_shadows,
    },
    wgpu::RenderState,
};

pub struct RenderFrameObjects {
    pub terrain: Vec<Vec<DrawIndirectArgs>>,
    pub transparent_terrain: Vec<Vec<DrawIndirectArgs>>,
    pub entity_instances: HashMap<MeshId, Vec<MeshInstanceId>>,
}

pub fn render_world(
    render_state: &mut RenderState,
    encoder: &mut CommandEncoder,
    view: &wgpu::TextureView,
) {
    let (terrain_buffer_draw_calls, transparent_terrain_buffer_draw_calls) =
        create_chunk_draw_calls(render_state);

    let entity_instances_to_render = create_entities_draw_calls(render_state);

    let render_objects = RenderFrameObjects {
        terrain: terrain_buffer_draw_calls,
        transparent_terrain: transparent_terrain_buffer_draw_calls,
        entity_instances: entity_instances_to_render,
    };

    render_sun_shadows(render_state, encoder, &render_objects);

    //setup for render game
    let mut gbuffer_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.base_color_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.lighting_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.normal_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &render_state.material_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            }),
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &render_state.depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    gbuffer_render_pass.set_pipeline(&render_state.gbuffer_render_pipeline);
    gbuffer_render_pass.set_bind_group(0, &render_state.camera_bind_group, &[]);
    gbuffer_render_pass.set_bind_group(1, &render_state.sun_shadow_textures_bind_group, &[]);

    render_opaque_chunks(render_state, &render_objects, &mut gbuffer_render_pass);
    render_entities(render_state, &render_objects, &mut gbuffer_render_pass);

    drop(gbuffer_render_pass);

    let mut volumetric_lighting_render_pass =
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_state.volumetric_lighting_gbuffer_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

    volumetric_lighting_render_pass.set_pipeline(&render_state.volumetric_lighting_render_pipeline);
    volumetric_lighting_render_pass.set_bind_group(0, &render_state.gbuffers_bind_group, &[]);
    volumetric_lighting_render_pass.set_bind_group(1, &render_state.camera_bind_group, &[]);
    volumetric_lighting_render_pass.set_bind_group(
        2,
        &render_state.sun_shadow_textures_bind_group,
        &[],
    );
    volumetric_lighting_render_pass.draw(0..3, 0..1);

    drop(volumetric_lighting_render_pass);

    let mut composition_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                store: wgpu::StoreOp::Store,
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &render_state.depth_view,
            depth_ops: None,
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    composition_render_pass.set_pipeline(&render_state.composition_render_pipeline);
    composition_render_pass.set_bind_group(0, &render_state.gbuffers_bind_group, &[]);
    composition_render_pass.set_bind_group(1, &render_state.camera_bind_group, &[]);
    composition_render_pass.set_bind_group(2, &render_state.volumetric_lighting_bind_group, &[]);
    composition_render_pass.draw(0..3, 0..1);

    drop(composition_render_pass);

    let mut transparent_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Transparent Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                store: wgpu::StoreOp::Store,
                load: wgpu::LoadOp::Load,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &render_state.depth_view,
            depth_ops: None,
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    transparent_render_pass.set_pipeline(&render_state.transparent_render_pipeline);
    transparent_render_pass.set_bind_group(0, &render_state.gbuffers_bind_group, &[]);
    transparent_render_pass.set_bind_group(1, &render_state.camera_bind_group, &[]);

    //render transparent
    render_transparent_chunks(render_state, &render_objects, &mut transparent_render_pass);

    drop(transparent_render_pass);
}
