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
    _pad: f32, // ensures 16-byte alignment for arrays or buffers
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0) var depth_texture_lod0_view: texture_depth_2d;
@group(2) @binding(1) var depth_texture_lod0_samplier: sampler_comparison;
@group(2) @binding(2) var depth_texture_lod0_distance_samplier: sampler;
@group(2) @binding(3) var<uniform> depth_texture_lod0_camera: CameraUniform;

@group(2) @binding(4) var depth_texture_lod1_view: texture_depth_2d;
@group(2) @binding(5) var depth_texture_lod1_samplier: sampler_comparison;
@group(2) @binding(6) var depth_texture_lod1_distance_samplier: sampler;
@group(2) @binding(7) var<uniform> depth_texture_lod1_camera: CameraUniform;

@group(2) @binding(8) var depth_texture_lod2_view: texture_depth_2d;
@group(2) @binding(9) var depth_texture_lod2_samplier: sampler_comparison;
@group(2) @binding(10) var depth_texture_lod2_distance_samplier: sampler;
@group(2) @binding(11) var<uniform> depth_texture_lod2_camera: CameraUniform;

@group(2) @binding(12) var depth_texture_lod3_view: texture_depth_2d;
@group(2) @binding(13) var depth_texture_lod3_samplier: sampler_comparison;
@group(2) @binding(14) var depth_texture_lod3_distance_samplier: sampler;
@group(2) @binding(15) var<uniform> depth_texture_lod3_camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<i32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let pos = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0)
    );

    let uv = array<vec2<f32>, 3>(
        vec2(0.0, 1.0),
        vec2(2.0, 1.0),
        vec2(0.0, -1.0)
    );

    return VertexOutput(
        vec4(pos[vertex_index], 0.0, 1.0),
        uv[vertex_index],
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth: f32 = textureSample(depth_texture, depth_sampler, in.uv);

    let x = in.uv.x * 2.0 - 1.0;
    let y = (1.0 - in.uv.y) * 2.0 - 1.0; 

    let clip_pos = vec4<f32>(x, y, depth, 1.0);
    let world_pos_h = camera.inverted_view_proj * clip_pos;
    let world_pos = world_pos_h.xyz / world_pos_h.w;
    let camera_normal = normalize(camera.position.xyz - world_pos.xyz);
    let dist = distance(camera.position.xyz, world_pos.xyz);

    const fog_density: f32 = 0.04;
    const sun_scatter: f32 = 0.04;
    const ambient_scatter: f32 = 0.01;

    const step_size: f32 = 1.0;
    const max_steps: f32 = 64;

    let loops: i32 = i32(clamp(floor(dist / step_size),0,max_steps));
    let left_dist = dist - (f32(loops) * step_size);

    var left_in_sunlight = true;
    var fog_light: f32 = 0;
    var transmittance: f32 = 1.0;

    for (var i = 0; i < loops; i += 1) {
        let start_ray_position = vec4<f32>((world_pos) + (camera_normal * f32(i) * step_size), 1.0);

        let light_clip_pos = depth_texture_lod1_camera.view_proj * start_ray_position;
        let light_coords = light_clip_pos.xyz / light_clip_pos.w;    
        if abs(light_coords.x) < 1 && abs(light_coords.y) < 1 {//sun_shadow_size_relative
            let shadow_texture_uv = vec2(light_coords.x, light_coords.y * -1) * 0.5 + 0.5;//sun_shadow_size_relative
            let depth = light_coords.z;
            let ground_depth = textureSample(depth_texture_lod1_view, depth_texture_lod1_distance_samplier, shadow_texture_uv);
            
            if ground_depth + 0.01 > depth { // in sunlight
                let scatter = sun_scatter * step_size;
                fog_light += transmittance * scatter;
                left_in_sunlight = true;
            }else{
                left_in_sunlight = false;
            }
            //fog_light += transmittance * ambient_scatter * step_size;
            transmittance *= exp(-fog_density * step_size);
        }
    }

    //how much light is absorbed.
    transmittance *= exp(-fog_density * left_dist);
    if left_in_sunlight {
        fog_light += transmittance * sun_scatter * left_dist;
    }else{
        //fog_light += transmittance * ambient_scatter * left_dist;
    }


    var data = vec4<f32>(0.1,0.1,0.8,0.0);
    data += vec4<f32>(1.0,1.0,0.0,0.0) * (fog_light * 0.25);
    data.a = 1.0 - transmittance;

    return data;
}