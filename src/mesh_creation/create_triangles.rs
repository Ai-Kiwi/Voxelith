use crate::utils::{Color, Vertex, VoxelPosition};

pub enum TriangleSide {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

pub fn add_triangle(vertices : &mut Vec<Vertex>, side : TriangleSide, x_position : i32, y_position : i32, z_position : i32, lod : i32, color : Color) {
    let x_offset = x_position;
    let y_offset = y_position - lod;
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

    // Push vertices with color & normal
    for &pos in &positions {
        vertices.push(Vertex {
            position: pos,
            color: color.into(),
            extra: [255,0,0,0],           // or any color per face
        });
    }
}
