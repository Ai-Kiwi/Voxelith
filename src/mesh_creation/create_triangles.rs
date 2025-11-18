use crate::utils::{Color, Vertex, Vec3};

pub enum TriangleSide {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

pub fn add_triangle(vertices : &mut Vec<Vertex>, indices : &mut Vec<u32>, side : TriangleSide, x_position : f32, y_position : f32, z_position : f32, lod : f32, color : Color) {
    let offset = vertices.len() as u32; // 3
    let x_offset = x_position + 0.5;
    let y_offset = y_position + 0.5 - lod;
    let z_offset = z_position + 0.5;

    let x0: f32 = x_offset;
    let x1: f32 = x_offset + lod;
    let y0: f32 = y_offset;
    let y1: f32 = y_offset + lod;
    let z0: f32 = z_offset;
    let z1: f32 = z_offset + lod;

    let positions = match side {
        TriangleSide::Front => 
            vec![
                Vec3::new(x0, y0, z1),
                Vec3::new(x1, y0, z1),
                Vec3::new(x1, y1, z1),
                Vec3::new(x0, y1, z1),
            ]
        ,
        TriangleSide::Back => vec![
                Vec3::new(x1, y0, z0),
                Vec3::new(x0, y0, z0),
                Vec3::new(x0, y1, z0),
                Vec3::new(x1, y1, z0),
            ],
        TriangleSide::Left => vec![
                Vec3::new(x0, y0, z0),
                Vec3::new(x0, y0, z1),
                Vec3::new(x0, y1, z1),
                Vec3::new(x0, y1, z0),
            ],
        TriangleSide::Right => vec![
                Vec3::new(x1, y0, z1),
                Vec3::new(x1, y0, z0),
                Vec3::new(x1, y1, z0),
                Vec3::new(x1, y1, z1),
            ],
        TriangleSide::Top => vec![
                Vec3::new(x0, y1, z1),
                Vec3::new(x1, y1, z1),
                Vec3::new(x1, y1, z0),
                Vec3::new(x0, y1, z0),
            ],
        TriangleSide::Bottom => vec![
                Vec3::new(x0, y0, z0),
                Vec3::new(x1, y0, z0),
                Vec3::new(x1, y0, z1),
                Vec3::new(x0, y0, z1),
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

    // Add two triangles for this quad
    indices.extend([
        offset,
        offset + 1,
        offset + 2,
        offset,
        offset + 2,
        offset + 3,
    ]);
}
