use std::{ops::{Add, AddAssign, Div, Mul, Sub}, thread::sleep, time::Duration};

use bytemuck::{Pod, Zeroable};

use crate::render::types;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position :  Vec3,
    pub color : Color,
    pub extra : [u8; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Unorm8x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress + std::mem::size_of::<[u8; 4]>() as wgpu::BufferAddress ,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Unorm8x4,
                }
            ]
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vec2{
    pub x : f32, 
    pub y : f32, 
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vec3{
    pub x : f32, 
    pub y : f32, 
    pub z : f32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vec4{
    x : f32, 
    y : f32, 
    z : f32,
    w : f32
}

impl Vec2 {
    pub const fn new(x : f32, y : f32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl Vec3 {
    pub const fn new(x : f32, y : f32, z : f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn normalize(&self) -> Vec3 {
        let len: f32 = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();

        Vec3 {
            x : self.x / len,
            y : self.y / len,
            z : self.z / len
        }
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        return (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    pub fn length(&self) -> f32 {
        return ((self.x * self.x) + (self.y * self.y) + (self.z * self.z)).sqrt()
    }

    pub fn angle_between(&self, other: &Vec3) -> f32 {
        let dot = self.dot(other);
        let mag = self.length() * other.length();

        let angle = (dot / mag).clamp(-1.0, 1.0).acos();
        return angle
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 { 
            x: self.x + rhs.x, 
            y: self.y + rhs.y, 
            z: self.z + rhs.z
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}


impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Vec4 {
    pub const fn new(x : f32, y : f32, z : f32, w : f32) -> Vec4 {
        Vec4 { x, y, z, w }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Color {
    pub r : u8,
    pub g : u8,
    pub b : u8,
    pub a : u8
}

impl Color {
    pub const fn new(r : u8, g : u8, b : u8, a : u8) -> Color {
        Color { r, g, b, a }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices : Vec<Vertex>, 
    pub indices: Vec<u32>, 
}


pub fn raycast_test(start_position : Vec3, direction_normal : Vec3) -> impl Iterator<Item = Vec3> {

    let mut current_position : Vec3 = start_position;

    std::iter::from_fn(move || {
        let mut next_x = if current_position.x - start_position.x > 0.0 {
            current_position.x.ceil()
        }else{
            current_position.x.floor()
        };
        if next_x.fract() == 0.0 {
            if direction_normal.x > 0.0 {
                next_x += 1.0;
            }else{
                next_x -= 1.0;
            }
        }

        let mut next_y = if current_position.y - start_position.y > 0.0 {
            current_position.y.ceil()
        }else{
            current_position.y.floor()
        };
        if next_y.fract() == 0.0 {
            if direction_normal.y > 0.0 {
                next_y += 1.0;
            }else{
                next_y -= 1.0;
            }
        }

        let mut next_z = if current_position.z - start_position.z > 0.0 {
            current_position.z.ceil()
        }else{
            current_position.z.floor()
        };
        if next_z.fract() == 0.0 {
            if direction_normal.z > 0.0 {
                next_z += 1.0;
            }else{
                next_z -= 1.0;
            }
        }

        let dist_x = (next_x - current_position.x) / direction_normal.x;
        let dist_y = (next_y - current_position.y) / direction_normal.y;
        let dist_z = (next_z - current_position.z) / direction_normal.z;

        if dist_x <= dist_y && dist_x <= dist_z {
            current_position += direction_normal * dist_x;
        }else if dist_y <= dist_x && dist_y <= dist_z {
            current_position += direction_normal * dist_y;
        }else if dist_z <= dist_y && dist_z <= dist_x {
            current_position += direction_normal * dist_z;
        } 
        Some(current_position)
    })
}