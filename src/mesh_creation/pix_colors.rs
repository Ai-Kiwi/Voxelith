use crate::{game::pixel::PixelTypes, utils::Color};

fn hash_position(x: i32, y: i32, z: i32) -> f32 {
    let mut seed = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (z as u32).wrapping_mul(2147483647);

    seed = (seed ^ (seed >> 13)).wrapping_mul(1274126177);

    // convert to 0–1 float
    (seed as f32) / (u32::MAX as f32)
}

pub fn calculate_pix_color(pixel_type : PixelTypes,x : i32, y : i32, z : i32) -> Color {

    let code = hash_position(x, y, z);

    let colors: [u8; 4] = match pixel_type {
        PixelTypes::Air => [0, 0, 0, 0],
        PixelTypes::Grass => {
            if code > 0.75 {
                [24, 176, 21, 255]
            }else if code > 0.5 {
                [57, 189, 55, 255]
            }else if code > 0.25 {
                [16, 191, 13, 255]
            } else {
                [22, 153, 20, 255]
            }
        },
        PixelTypes::Dirt => {
            if code > 0.95 {
                [79, 76, 74, 255]
            }else if code > 0.75 {
                [92, 58, 17, 255]
            }else if code > 0.5 {
                [82, 49, 11, 255]
            }else if code > 0.25 {
                [99, 59, 13, 255]
            } else {
                [117, 71, 19, 255]
            }
        },
        PixelTypes::Stone => [67, 71, 66, 255],
        PixelTypes::Water => [27, 38, 161, 200],
        PixelTypes::Sand => {
            if code > 0.75 {
                [227, 177, 77, 255]
            }else if code > 0.5 {
                [201, 152, 52, 255]
            }else if code > 0.25 {
                [191, 141, 38, 255]
            } else {
                [222, 157, 22, 255]
            }
        },
        PixelTypes::Snow => {
            if code > 0.95 {
                [185, 220, 235, 255]
            }else if code > 0.5 {
                [230, 237, 232, 255]
            }else {
                [228, 237, 230, 255]
            }
        },
    };

    Color { 
        r: colors[0], 
        g: colors[1], 
        b: colors[2], 
        a: colors[3] 
    }
}