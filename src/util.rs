use glam::{Mat4, Vec2, Vec3};
use std::{mem, slice};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device};

#[repr(C)]
pub struct VSInput {
    position: Vec3,
    uv: Vec2
}

impl VSInput {
    pub fn new(position: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            uv
        }
    }
}

pub fn create_uniform_buffer(device: &Device, size: usize) -> Buffer {
    let transform = Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5));

    device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: unsafe {
            slice::from_raw_parts(
                &transform as *const Mat4 as *const _,
                mem::size_of::<Mat4>(),
            )
        },
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

pub fn create_vertex_buffer(device: &Device) -> Buffer {
    let vertices = [VSInput { position: Vec3::new(-0.8, -0.5, -1.0), uv: Vec2::new(0., 1.) },
                            VSInput { position: Vec3::new(0.5, -0.5, -1.0), uv: Vec2::new(1., 1.) },
                             VSInput { position: Vec3::new(-0.8, 0.5, -1.0), uv: Vec2::new(0., 0.,) },
                             VSInput { position: Vec3::new(0.5, 0.5, -1.0), uv: Vec2::new(1., 0.) }];

    device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: unsafe { slice::from_raw_parts(vertices.as_ptr().cast(), vertices.len() * mem::size_of::<VSInput>()) },
        usage: BufferUsages::VERTEX,
    })
}

pub fn create_index_buffer(device: &Device) -> Buffer {
    let indices: [u32; 6] = [0, 1, 2, 1, 2, 3];

    device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: unsafe { slice::from_raw_parts(indices.as_ptr().cast(), indices.len() * mem::size_of::<u32>())},
        usage: BufferUsages::INDEX,
    })
}