use std::num::NonZeroU32;
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Device, Extent3d, Queue, ShaderStages, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};
use wgpu::util::DeviceExt;

pub struct Texture2D {
    pub texture: Texture,
    pub view: TextureView
}

impl Texture2D {
    pub fn new(device: &Device, queue: &Queue, path: &str) -> Self {
        let image = image::open(path).expect(&format!("Failed to load {}", path))
            .to_rgba8();

        let texture = device.create_texture_with_data(queue, &TextureDescriptor {
            label: None,
            size: Extent3d { width: image.width(), height: image.height(), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING,
        }, &image);

        let view= texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(TextureFormat::Rgba8UnormSrgb),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(1),
        });

        Self {
            texture,
            view
        }
    }
}