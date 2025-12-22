@group(0) @binding(0) var base_color_texture: texture_2d<f32>;
@group(0) @binding(1) var base_color_sampler: sampler;

@group(0) @binding(2) var lighting_texture: texture_2d<f32>;
@group(0) @binding(3) var lighting_sampler: sampler;

@group(0) @binding(4) var normal_texture: texture_2d<f32>;
@group(0) @binding(5) var normal_sampler: sampler;

@group(0) @binding(6) var material_texture: texture_2d<f32>;
@group(0) @binding(7) var material_sampler: sampler;

@group(0) @binding(8) var depth_texture: texture_depth_2d;
@group(0) @binding(9) var depth_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<i32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let pos = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0)
    );

    return VertexOutput(
        vec4(pos[vertex_index], 0.0, 1.0),
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.position.xy / vec2<f32>(textureDimensions(base_color_texture, 0));

    let color = textureSample(base_color_texture, base_color_sampler, uv);
    let lighting = textureSample(lighting_texture, lighting_sampler, uv);
    let normal = textureSample(normal_texture, normal_sampler, uv);
    let material = textureSample(material_texture, material_sampler, uv);
    let depth = textureSample(depth_texture, depth_sampler, uv);


    let base_color = vec4<f32>(color);
    var final_color = vec4<f32>(0.0);

    final_color += base_color * lighting.r; //lighting light
    final_color += base_color * lighting.g; //sun light

    final_color = final_color;

    return final_color;
}

 