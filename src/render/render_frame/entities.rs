use std::collections::HashMap;

use crate::render::{
    entity_meshs::{MeshId, MeshInstanceId, MeshInstanceRaw},
    render_frame::render_world::RenderFrameObjects,
    wgpu::RenderState,
};

pub fn create_entities_draw_calls(
    render_state: &mut RenderState,
) -> HashMap<MeshId, Vec<MeshInstanceId>> {
    let mut entity_instances_to_render: HashMap<MeshId, Vec<MeshInstanceId>> = HashMap::new();
    for entity in &render_state.entities {
        let mesh_id: MeshId = entity.render_mesh_id;
        let instance_id = entity.instance_id;
        match entity_instances_to_render.get_mut(&mesh_id) {
            Some(draw_calls) => {
                draw_calls.push(instance_id);
            }
            None => {
                let mut buffer_calls: Vec<MeshInstanceId> = Vec::new();
                buffer_calls.push(instance_id);
                entity_instances_to_render.insert(mesh_id, buffer_calls);
            }
        }
    }
    //put the updated data into buffers
    for (mesh_id, instance_buffer) in &render_state.mesh_instances {
        if let Some(entity_list) = entity_instances_to_render.get(&mesh_id) {
            let mut buffer_contents: Vec<u8> =
                Vec::with_capacity(entity_list.len() * size_of::<MeshInstanceRaw>());
            for entity_instance_id in entity_list {
                let instance_data = instance_buffer
                    .mesh_instances
                    .get(entity_instance_id)
                    .unwrap();
                buffer_contents.extend_from_slice(bytemuck::bytes_of(&instance_data.to_raw()));
            }
            render_state
                .queue
                .write_buffer(&instance_buffer.instances_buffer, 0, &buffer_contents);
        }
    }

    return entity_instances_to_render;
}

pub fn render_entities(
    render_state: &RenderState,
    render_frame_objects: &RenderFrameObjects,
    render_pass: &mut wgpu::RenderPass<'_>,
) {
    for mesh in &render_frame_objects.entity_instances {
        let buffer_info = render_state.mesh_instances.get(&mesh.0).unwrap();
        let vertex_info = render_state.mesh_id_reference.get(&mesh.0).expect(
            format!(
                "Failed to render entity as texture id {} is not loaded",
                mesh.0.0
            )
            .as_str(),
        );

        render_pass.set_vertex_buffer(0, render_state.entity_meshs_buffer.slice(..));
        render_pass.set_vertex_buffer(1, buffer_info.instances_buffer.slice(..));

        render_pass.draw(
            vertex_info.start..(vertex_info.start + vertex_info.length),
            0..(mesh.1.len() as u32),
        );
    }
}
