use crate::utils::{Color, Material, Vertex, VoxelPosition};

pub enum TriangleSide {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

pub fn add_triangle(vertices : &mut Vec<Vertex>, side : TriangleSide, x_position : i32, y_position : i32, z_position : i32, lod : i32, color : &Color, material : &Material) {
    let x_offset = x_position;
    let y_offset = y_position;
    let z_offset = z_position;

    let x0: i32 = x_offset;
    let x1: i32 = x_offset + lod;
    let y0: i32 = y_offset;
    let y1: i32 = y_offset + lod;
    let z0: i32 = z_offset;
    let z1: i32 = z_offset + lod;

    let positions = match side {
        TriangleSide::Front => 
            vec![
                VoxelPosition::new(x0, y0, z1),
                VoxelPosition::new(x1, y0, z1),
                VoxelPosition::new(x1, y1, z1),
                VoxelPosition::new(x0, y0, z1),
                VoxelPosition::new(x1, y1, z1),
                VoxelPosition::new(x0, y1, z1),
            ]
        ,
        TriangleSide::Back => vec![
                VoxelPosition::new(x1, y0, z0),
                VoxelPosition::new(x0, y0, z0),
                VoxelPosition::new(x0, y1, z0),
                VoxelPosition::new(x1, y0, z0),
                VoxelPosition::new(x0, y1, z0),
                VoxelPosition::new(x1, y1, z0),
            ],
        TriangleSide::Left => vec![
                VoxelPosition::new(x0, y0, z0),
                VoxelPosition::new(x0, y0, z1),
                VoxelPosition::new(x0, y1, z1),
                VoxelPosition::new(x0, y0, z0),
                VoxelPosition::new(x0, y1, z1),
                VoxelPosition::new(x0, y1, z0),
            ],
        TriangleSide::Right => vec![
                VoxelPosition::new(x1, y0, z1),
                VoxelPosition::new(x1, y0, z0),
                VoxelPosition::new(x1, y1, z0),
                VoxelPosition::new(x1, y0, z1),
                VoxelPosition::new(x1, y1, z0),
                VoxelPosition::new(x1, y1, z1),
            ],
        TriangleSide::Top => vec![
                VoxelPosition::new(x0, y1, z1),
                VoxelPosition::new(x1, y1, z1),
                VoxelPosition::new(x1, y1, z0),
                VoxelPosition::new(x0, y1, z1),
                VoxelPosition::new(x1, y1, z0),
                VoxelPosition::new(x0, y1, z0),
            ],
        TriangleSide::Bottom => vec![
                VoxelPosition::new(x0, y0, z0),
                VoxelPosition::new(x1, y0, z0),
                VoxelPosition::new(x1, y0, z1),
                VoxelPosition::new(x0, y0, z0),
                VoxelPosition::new(x1, y0, z1),
                VoxelPosition::new(x0, y0, z1),
            ],
    };

    let side_value = match side {
        TriangleSide::Top => 0,
        TriangleSide::Bottom => 1,
        TriangleSide::Left => 2,
        TriangleSide::Right => 3,
        TriangleSide::Front => 4,
        TriangleSide::Back => 5,
    };

    // Push vertices with color & normal
    for &pos in &positions {
        vertices.push(Vertex {
            position: pos,
            color: color.clone(),
            extra: [material.reflectiveness,material.roughness,material.metallicness,side_value],           // or any color per face
        });
    }
}
