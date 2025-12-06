// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0) var voxel_map_texture: texture_3d<u32>;

struct VertexInput {
    @location(0) position: vec3<i32>,
    @location(1) color: vec4<f32>,
    @location(2) extra: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec4<f32>,
    @location(2) normal: vec3<f32>,
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

fn test_raycast(origin : vec3<f32>, direction : vec3<f32>, final_distance : u32) -> u32 {
    var distance: u32 = 0;
    var ray_position = origin;
    ray_position = ray_position + (direction / 100);
    for (var i: u32 = 0; i < final_distance; i = i + 1) {
        let ray_voxel_x = i32(trunc(ray_position.x));
        let ray_voxel_y = i32(trunc(ray_position.y));
        let ray_voxel_z = i32(trunc(ray_position.z));
        
        let value = textureLoad(voxel_map_texture, vec3<i32>(ray_voxel_x, ray_voxel_y, ray_voxel_z), 0);
        if value.r != 0 {
            distance = i + 1;
            break;
        }
        ray_position = next_ray_position(ray_position, direction);
    };
    return distance;
}   



@fragment
fn fs_main(in: VertexOutput) -> GbufferOutput {

    var gbuffers = GbufferOutput();
    gbuffers.base_color = in.color;
    gbuffers.normal = vec4<f32>(in.normal, 0.0);
    gbuffers.material = vec4<f32>(gbuffers.material.r, gbuffers.material.g, gbuffers.material.b, 0.0);

    //distance to origin which we are placing light
    const light_info = vec4<f32>(0.0,5,0.0,25);

    let diff = vec3<f32>(light_info.x - in.world_pos.x, light_info.y - in.world_pos.y, light_info.z - in.world_pos.z);
    let distance = length(diff);
    let intensity = light_info.a / ((distance + 0.01) * (distance + 0.01));
    gbuffers.lighting = vec4<f32>(0.0);

    gbuffers.lighting.r = intensity; //lighting
    
    let light_relative = normalize(light_info.xyz - in.world_pos.xyz);
    let camera_relative = normalize(light_info.xyz + camera.position.xyz);
    let merged_relative = normalize(light_relative + camera_relative);


    let ambient = 0.25;
    let diffuse = 1.0 * max(dot(in.normal, light_relative), 0.0);
    let specular = 1.0 * pow(max(dot(in.normal, merged_relative), 0.0), 15.0);




    gbuffers.lighting.r = (ambient + diffuse + specular) * intensity;

    
    
    const sun_direction = vec3<f32>(0.1,0.25,0.05);
    let shadow_distance = test_raycast(vec3<f32>(in.world_pos.x,in.world_pos.y,in.world_pos.z), sun_direction, 25);
    if shadow_distance == 0 {
        //gbuffers.lighting.g = 1.0; //sun
    }else{
        gbuffers.lighting.g = 0; //sun
    }

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