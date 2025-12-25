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

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inverted_view_proj: mat4x4<f32>,
    position: vec3<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

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

fn get_pixel_data(uv : vec2<f32>) -> vec4<f32> {
    let screen_dimensions = textureDimensions(depth_texture);
    let width = f32(screen_dimensions.x);
    let height = f32(screen_dimensions.y);

    let color = textureSample(base_color_texture, base_color_sampler, uv);
    let lighting = textureSample(lighting_texture, lighting_sampler, uv);
    let depth = textureSample(depth_texture, depth_sampler, uv);

    let x = uv.x * 2.0 - 1.0;
    let y = (1.0 - uv.y) * 2.0 - 1.0; 
    let clip_pos = vec4<f32>(x, y, depth, 1.0);
    let world_pos_h = camera.inverted_view_proj * clip_pos;
    let world_pos = world_pos_h.xyz / world_pos_h.w;

    let dist = distance(camera.position, world_pos);

    let camera_normal = normalize(world_pos - camera.position);

    let base_color = vec4<f32>(color);
    var final_color = vec4<f32>(0.0);

    final_color += base_color * lighting.r; //lighting light
    final_color += base_color * lighting.g; //sun light

    let sky_color = vec4((camera_normal.y + 0.5) / 4,(camera_normal.y + 0.5) / 4,1.0,1.0);

    let sky_strength = clamp((dist - 5000) / 100, 0, 1);

    final_color = mix(final_color, sky_color, sky_strength);

    return final_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.position.xy / vec2<f32>(textureDimensions(base_color_texture, 0));
    
    let depth = textureSample(depth_texture, depth_sampler, uv);
    let normal = textureSample(normal_texture, normal_sampler, uv);
    let material = textureSample(material_texture, material_sampler, uv);


    let x = uv.x * 2.0 - 1.0;
    let y = (1.0 - uv.y) * 2.0 - 1.0; 
    let clip_pos = vec4<f32>(x, y, depth, 1.0);
    let world_pos_h = camera.inverted_view_proj * clip_pos;
    var world_ray_position = world_pos_h.xyz / world_pos_h.w;

    let view_dir = normalize(camera.position - world_ray_position);
    let reflect_dir = reflect(-view_dir, normal.xyz);

    var pixel_color_data = get_pixel_data(uv);
    
    if material.b > 0 && false == true { //disabled for now
        var i = 0;
        loop {
            world_ray_position = world_ray_position + (reflect_dir * 0.1);
            i = (i + 1);
            if ((i > 150)) {
                break;
            }
            let clip_pos = camera.view_proj * vec4(world_ray_position, 1.0);
            let screen_coords = clip_pos.xyz / clip_pos.w;    
            let ray_depth = screen_coords.z * 0.5 + 0.5;

            let screen_coords_uv = vec2(screen_coords.x, screen_coords.y * -1) * 0.5 + 0.5;
            let screen_depth = textureSample(depth_texture, depth_sampler, screen_coords_uv);

            if (any(screen_coords_uv < vec2(0.0)) || any(screen_coords_uv > vec2(1.0))) {
                break;
            }

            if ray_depth > screen_depth - 0.001 {
                pixel_color_data = get_pixel_data(screen_coords_uv);
                break;
            }
        }
    }
    return pixel_color_data;
}