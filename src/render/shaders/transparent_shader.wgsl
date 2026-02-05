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

// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    inverted_view_proj: mat4x4<f32>,
    position: vec3<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<i32>,
    @location(1) color: vec4<f32>,
    @location(2) extra: vec4<f32>, //reflectiveness, roughness, metalic. Normal
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) extra: vec4<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.extra = model.extra;
    out.color = model.color;
    var position = vec3<f32>(model.position);
    //out.position = vec4<f32>(position, 1.0);
    
    let diff = vec2<f32>(position.x - camera.position.x, position.z - camera.position.z);
    let distance = length(diff);

    let normal_index = i32(model.extra.a * 255);
    var normal = vec3<f32>(0.0);
    switch(normal_index){
        case 0: {
            normal = vec3<f32>(0.0,1.0,0.0);
        }
        case 1: {
            normal = vec3<f32>(0.0,-1.0,0.0);
        }
        case 2: {
            normal = vec3<f32>(-1.0,0.0,0.0);
        }
        case 3: {
            normal = vec3<f32>(1.0,0.0,0.0);
        }
        case 4: {
            normal = vec3<f32>(0.0,0.0,1.0);
        }
        case 5: {
            normal = vec3<f32>(0.0,0.0,-1.0);
        }
        default: {
            normal = vec3<f32>(0.0,1.0,0.0);
        }
    };
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    //position.y = position.y - (distance / 2.5);

    let pos_f32: vec3<f32> = vec3<f32>(position); 
    out.world_pos = model_matrix * vec4<f32>(pos_f32, 1.0);
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(pos_f32, 1.0);
    out.normal = normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixel_color_data = in.color;
    
    

    return pixel_color_data;
}