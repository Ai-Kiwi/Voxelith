// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    inverted_view_proj: mat4x4<f32>,
    position: vec3<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<i32>,
    @location(1) color: vec4<f32>,
    @location(2) extra: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var position = vec3<f32>(model.position);

    let pos_f32: vec3<f32> = vec3<f32>(position); 
    out.clip_position = camera.view_proj * vec4<f32>(pos_f32, 1.0);
    return out;
}

@fragment
fn fs_main() {}

 