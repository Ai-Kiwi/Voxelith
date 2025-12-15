use crate::{game::pixel::PixelTypes, utils::{Color, Material}};

fn hash_position(x: i32, y: i32, z: i32) -> f32 {
    let mut seed = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (z as u32).wrapping_mul(2147483647);

    seed = (seed ^ (seed >> 13)).wrapping_mul(1274126177);

    // convert to 0–1 float
    (seed as f32) / (u32::MAX as f32)
}

pub fn calculate_pix_color(pixel_type : PixelTypes,x : i32, y : i32, z : i32) -> (Color, Material) {

    let code = hash_position(x, y, z);

    let pixel_data: (Color, Material) = match pixel_type {
        PixelTypes::Air => (Color::new(0, 0, 0, 0),Material::new(0,0,0)),
        PixelTypes::Grass => {
            if code > 0.75 {
                (Color::new(24, 176, 21, 255),Material::new(0,0,0))
            }else if code > 0.5 {
                (Color::new(57, 189, 55, 255),Material::new(0,0,0))
            }else if code > 0.25 {
                (Color::new(16, 191, 13, 255),Material::new(0,0,0))
            } else {
                (Color::new(22, 153, 20, 255),Material::new(0,0,0))
            }
        },
        PixelTypes::Dirt => {
            if code > 0.95 {
                (Color::new(79, 76, 74, 255),Material::new(5,230,0))
            }else if code > 0.75 {
                (Color::new(92, 58, 17, 255),Material::new(5,225,0))
            }else if code > 0.5 {
                (Color::new(82, 49, 11, 255),Material::new(4,217,0))
            }else if code > 0.25 {
                (Color::new(99, 59, 13, 255),Material::new(5,222,0))
            } else {
                (Color::new(117, 71, 19, 255),Material::new(6,225,0))
            }
        },
        PixelTypes::Stone => {
            if code > 0.75 {
                (Color::new(50, 55, 50, 255),Material::new(0,0,0))
            }else if code > 0.5 {
                (Color::new(66, 69, 60, 255),Material::new(0,0,0))
            }else if code > 0.25 {
                (Color::new(63, 65, 60, 255),Material::new(0,0,0))
            } else {
                (Color::new(67, 71, 66, 255),Material::new(0,0,0))
            }
        },
        PixelTypes::Water => (Color::new(27, 38, 161, 200),Material::new(0,0,0)),
        PixelTypes::Sand => {
            if code > 0.75 {
                (Color::new(227, 177, 77, 255), Material::new(10, 200, 0))
            } else if code > 0.5 {
                (Color::new(201, 152, 52, 255), Material::new(8, 210, 0))
            } else if code > 0.25 {
                (Color::new(191, 141, 38, 255), Material::new(6, 215, 0))
            } else {
                (Color::new(222, 157, 22, 255), Material::new(12, 195, 0))
            }
        },
        PixelTypes::Snow => {
            if code > 0.95 {
                (Color::new(185, 220, 235, 255),Material::new(0,0,0))
            }else if code > 0.5 {
                (Color::new(230, 237, 232, 255),Material::new(0,0,0))
            }else {
                (Color::new(228, 237, 230, 255),Material::new(0,0,0))
            }
        },
    };

    return pixel_data;
}