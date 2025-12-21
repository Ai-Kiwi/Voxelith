// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0) var depth_texture_lod0_view: texture_depth_2d;
@group(1) @binding(1) var depth_texture_lod0_samplier: sampler_comparison;
@group(1) @binding(2) var<uniform> depth_texture_lod0_camera: CameraUniform;

@group(1) @binding(3) var depth_texture_lod1_view: texture_depth_2d;
@group(1) @binding(4) var depth_texture_lod1_samplier: sampler_comparison;
@group(1) @binding(5) var<uniform> depth_texture_lod1_camera: CameraUniform;

@group(1) @binding(6) var depth_texture_lod2_view: texture_depth_2d;
@group(1) @binding(7) var depth_texture_lod2_samplier: sampler_comparison;
@group(1) @binding(8) var<uniform> depth_texture_lod2_camera: CameraUniform;

@group(1) @binding(9) var depth_texture_lod3_view: texture_depth_2d;
@group(1) @binding(10) var depth_texture_lod3_samplier: sampler_comparison;
@group(1) @binding(11) var<uniform> depth_texture_lod3_camera: CameraUniform;

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

struct GbufferOutput {
    @location(0) base_color: vec4<f32>,
    @location(1) lighting: vec4<f32>,
    @location(2) normal: vec4<f32>,
    @location(3) material: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
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

    //position.y = position.y - (distance / 2.5);

    let pos_f32: vec3<f32> = vec3<f32>(position); 
    out.world_pos = vec4<f32>(pos_f32, 1.0);
    out.clip_position = camera.view_proj * vec4<f32>(pos_f32, 1.0);
    out.normal = normal;
    return out;
}

fn next_ray_position(ray : vec3<f32>, direction : vec3<f32>) -> vec3<f32> {
    var step_x: f32;
    if direction.x > 0.0 {
        step_x = 1;
    } else {
        step_x = -1;
    }

    var step_y: f32;
    if direction.y > 0.0 {
        step_y = 1;
    } else {
        step_y = -1;
    }

    var step_z: f32;
    if direction.z > 0.0 {
        step_z = 1;
    } else {
        step_z = -1;
    }

    let next_x = trunc(ray.x + step_x);
    let next_y = trunc(ray.y + step_y);
    let next_z = trunc(ray.z + step_z);

    let distance_x = (next_x - ray.x) / direction.x;
    let distance_y = (next_y - ray.y) / direction.y;
    let distance_z = (next_z - ray.z) / direction.z;


    if distance_x <= distance_y && distance_x <= distance_z {
        return ray + (direction * distance_x);
    }else if distance_y <= distance_x && distance_y <= distance_z {
        return ray + (direction * distance_y);
    }else if distance_z <= distance_y && distance_z <= distance_x {
        return ray + (direction * distance_z);
    }
    return ray + (direction * distance_z);
}


@fragment
fn fs_main(in: VertexOutput) -> GbufferOutput {

    var gbuffers = GbufferOutput();
    gbuffers.base_color = in.color;
    gbuffers.normal = vec4<f32>(in.normal, 0.0);
    gbuffers.material = vec4<f32>(in.extra.r, in.extra.g, in.extra.b, 0.0);//reflectiveness, roughness, metallicness. Normal
    let reflectiveness = in.extra.r;
    let roughness = in.extra.g;
    let metallicness = in.extra.b;

    //distance to origin which we are placing light
    gbuffers.lighting.r = 0;
    for (var i: i32 = 0; i < 1; i++) {
        const light_info = vec4<f32>(0.0,25,0.0,10000);

        let diff = vec3<f32>(light_info.x - in.world_pos.x, light_info.y - in.world_pos.y, light_info.z - in.world_pos.z);
        let distance = length(diff);
        let intensity = light_info.a / ((distance + 0.01) * (distance + 0.01));
        gbuffers.lighting = vec4<f32>(0.0);

        gbuffers.lighting.r = intensity; //lighting
        
        let light_relative = normalize(light_info.xyz - in.world_pos.xyz);
        let camera_relative = normalize(light_info.xyz + camera.position.xyz);
        let merged_relative = normalize(light_relative + camera_relative);

        let shininess = mix(256.0, 2.0, roughness);
        
        let ambient = 0.25;
        let diffuse = (1.0 - metallicness) * max(dot(in.normal, light_relative), 0.0);
        let specular_strength = mix(0.04, 1.0, metallicness);
        let specular = specular_strength * pow(max(dot(in.normal, merged_relative), 0.0), 15.0);
        
        gbuffers.lighting.r += (ambient + diffuse + specular) * intensity;
    }
    
    //test shadow

    //get camera distnce to point to pick shadow
    let shadow_camera_diff = vec3<f32>(0 - in.world_pos.x, 0 - in.world_pos.y, 0- in.world_pos.z);
    let shadow_camera_distance = length(shadow_camera_diff);

    let light_clip_pos = depth_texture_lod0_camera.view_proj * in.world_pos;
    let light_coords = light_clip_pos.xyz / light_clip_pos.w;
    
    gbuffers.lighting.r = 1;
    gbuffers.lighting.g = light_coords.x;
    

    var closeness_response = 0.0;
    if abs(light_coords.x) < 1 && abs(light_coords.y) < 1 {
        let shadow_texture_uv = vec2(light_coords.x, light_coords.y * -1) * 0.5 + 0.5;
        let depth = light_coords.z - 0.001;
        closeness_response = textureSampleCompare(
            depth_texture_lod0_view,
            depth_texture_lod0_samplier,
            shadow_texture_uv,
            depth
        );
    }else if abs(light_coords.x) < 3 && abs(light_coords.y) < 3 {
        let shadow_texture_uv = vec2(light_coords.x / 3, light_coords.y * -1 / 3) * 0.5 + 0.5;
        let depth = light_coords.z - 0.001;
        closeness_response = textureSampleCompare(
            depth_texture_lod1_view,
            depth_texture_lod1_samplier,
            shadow_texture_uv,
            depth
        );
    }else if abs(light_coords.x) < 8 && abs(light_coords.y) < 8 {
        let shadow_texture_uv = vec2(light_coords.x / 8, light_coords.y * -1 / 8) * 0.5 + 0.5;
        let depth = light_coords.z - 0.001;
        closeness_response = textureSampleCompare(
            depth_texture_lod2_view,
            depth_texture_lod2_samplier,
            shadow_texture_uv,
            depth
        );
    }else if abs(light_coords.x) < 24 && abs(light_coords.y) < 24 {
        let shadow_texture_uv = vec2(light_coords.x / 24, light_coords.y * -1 / 24) * 0.5 + 0.5;
        let depth = light_coords.z - 0.001;
        closeness_response = textureSampleCompare(
            depth_texture_lod3_view,
            depth_texture_lod3_samplier,
            shadow_texture_uv,
            depth
        );
    }else{
        closeness_response = 0;
    }


    if closeness_response < 0.5 {
        gbuffers.lighting.g = 0;
    }else{
        gbuffers.lighting.g = 1;
    }

    let light_relative = normalize(depth_texture_lod0_camera.position.xyz - in.world_pos.xyz);
    let camera_relative = normalize(depth_texture_lod0_camera.position.xyz + camera.position.xyz);
    let merged_relative = normalize(light_relative + camera_relative);

    let shininess = mix(256.0, 2.0, roughness);
    
    let diffuse = (1.0 - metallicness) * max(dot(in.normal, light_relative), 0.0);
    let specular_strength = mix(0.04, 1.0, metallicness);
    let specular = specular_strength * pow(max(dot(in.normal, merged_relative), 0.0), 15.0);
    
    gbuffers.lighting.g = (diffuse + specular) * closeness_response;


    
    return gbuffers;
}

//ambient lighting which uses teardowns approch
//volumentic noise some how
//transperncy
//water effects
//metalic effects
//reflective effects
//ambiant occulstion

//particle system

//leaving room then going into another should change screen brightness like your eyes adjusting

//look at adding fog and mist effects into the game

//reflections base on screen contents. if the content is no on the screen then switch to using a blacked out version


//lighting might use shadows depends how it works out in practice