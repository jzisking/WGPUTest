use std::{mem, slice};
use wgpu::{Buffer, BufferUsages, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::util::VSInput;

pub struct BufferBuilder {
    vertices: Vec<VSInput>,
    indices: Vec<u32>
}

impl BufferBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }

    pub fn add_quad(&mut self, tl: VSInput, tr: VSInput, br: VSInput, bl: VSInput) -> &mut Self {
        let offset = self.vertices.len() as u32;

        self.vertices.push(tl);
        self.vertices.push(tr);
        self.vertices.push(br);
        self.vertices.push(bl);

        self.indices.push(offset + 0);
        self.indices.push(offset + 1);
        self.indices.push(offset + 2);

        self.indices.push(offset + 2);
        self.indices.push(offset + 3);
        self.indices.push(offset + 0);

        self
    }

    pub fn build(&mut self, device: &Device) -> (Buffer, Buffer, u32) {
        let v = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: unsafe { slice::from_raw_parts(self.vertices.as_ptr().cast(), self.vertices.len() * mem::size_of::<VSInput>())},
            usage: BufferUsages::VERTEX,
        });

        let i = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: unsafe { slice::from_raw_parts(self.indices.as_ptr().cast(), self.indices.len() * mem::size_of::<u32>())},
            usage: BufferUsages::INDEX,
        });

        (v, i, self.indices.len() as _)
    }
}