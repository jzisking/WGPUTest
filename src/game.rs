use crate::{util, world};
use futures_lite::future;
use glam::{Mat4, Quat, Vec3};
use std::borrow::Cow;
use std::{mem, slice};
use std::collections::HashSet;
use std::mem::swap;
use std::num::NonZeroU32;
use dolly::drivers::YawPitch;
use dolly::prelude::{CameraRig, Position, Smooth};
use wgpu::{include_wgsl, Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, BufferSize, Color, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, FragmentState, Instance, Limits, LoadOp, Operations, PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, RequestAdapterOptionsBase, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration, TextureUsages, TextureViewDescriptor, VertexState, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat, BufferSlice, IndexFormat, TextureSampleType, TextureViewDimension, SamplerBindingType, SamplerDescriptor, Sampler, BufferAddress, RenderPassDepthStencilAttachment, TextureDescriptor, Extent3d, TextureDimension, TextureFormat, TextureAspect, Texture, TextureView, DepthStencilState, CompareFunction, PrimitiveState, PolygonMode};
use winit::dpi::{PhysicalSize, Size};
use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};
use crate::texture::Texture2D;
use crate::util::VSInput;
use crate::world::{BLOCK_TYPE_AIR, BLOCK_TYPE_STONE, Chunk, ChunkBuilder, World};

pub struct Game {
    event_loop: EventLoop<()>,
    window: Window,

    //WGPU
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    surface_config: SurfaceConfiguration,

    shader_module: ShaderModule,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    texture: Texture2D,
    sampler: Sampler,
    camera_rig: CameraRig,

    depth: Texture,
    depth_view: TextureView,

    world: World,
    index_count: u32
}

impl Game {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("TestEngine".to_owned())
            .with_inner_size(Size::Physical(PhysicalSize::new(2560, 1440)))
            .build(&event_loop)
            .expect("Failed to create window");

        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = future::block_on(async {
            instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: Default::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .await
        })
        .expect("Failed to find compatible adapter");

        let (device, queue) = future::block_on(async {
            adapter
                .request_device(
                    &DeviceDescriptor {
                        label: None,
                        features: wgpu::Features::POLYGON_MODE_LINE,
                        limits: Limits::default(),
                    },
                    None,
                )
                .await
        })
        .expect("Failed to create device and queue");

        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: Default::default(),
            alpha_mode: CompositeAlphaMode::Opaque,
        };

        let shader_module = device.create_shader_module(include_wgsl!("shaders/default.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(mem::size_of::<Mat4>() as _),
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }, BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            }],
        });

        let uniform_buffer = util::create_uniform_buffer(&device, mem::size_of::<Mat4>());

        let texture = Texture2D::new(&device, &queue, "texture.png");
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mipmap_filter: Default::default(),
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: BufferSize::new(mem::size_of::<Mat4>() as _),
                }),
            }, BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture.view),
            }, BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(&sampler),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<VSInput>() as _,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }, VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: mem::size_of::<Vec3>() as _,
                        shader_location: 1,
                    }],
                }],
            },
            primitive: PrimitiveState {
                topology: Default::default(),
                strip_index_format: None,
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            multiview: None,
        });

        let camera_rig = CameraRig::builder()
            .with(Position::new(Vec3::Y))
            .with(YawPitch::new())
            .with(Smooth::new_position_rotation(0., 0.))
            .build();

        surface.configure(&device, &surface_config);

        let chunk = Chunk::new(world::Position::default(), BLOCK_TYPE_STONE);

        let mut world = World::new();
        world.add_chunk(chunk);

        let mut chunk_builder = ChunkBuilder::new(world.chunks.get(&world::Position::default()).unwrap());
        let (vertex_buffer, index_buffer, index_count) = chunk_builder.build_mesh(&device, &world);

        let depth = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: window.inner_size().width,
                height: window.inner_size().height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
        });
        
        let depth_view = depth.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(TextureFormat::Depth24Plus),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(1),
        });

        Self {
            event_loop,
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            shader_module,
            bind_group_layout,
            pipeline_layout,
            uniform_buffer,
            bind_group,
            pipeline,
            vertex_buffer,
            index_buffer,
            texture,
            sampler,
            camera_rig,
            world,
            index_count,
            depth,
            depth_view
        }
    }

    fn render(&mut self) {
        let transform = self.camera_rig.final_transform;

        let mut projection_matrix = Mat4::perspective_lh(90., 16. / 9., 0.1, 1000.);
        let vp = projection_matrix * Mat4::look_at_lh(transform.position,
            transform.position + transform.forward(), transform.up());

        self.queue.write_buffer(&self.uniform_buffer, 0,
            unsafe { slice::from_raw_parts(&vp as *const Mat4 as *const _, mem::size_of::<Mat4>())});

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to get current frame");

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::GREEN),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn run(mut self) {
        let mut running = true;

        let delta_time = 1. / 120.; //TODO: bad

        let mut pressed_keys = HashSet::new();

        while running {
            self.event_loop.run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::WindowEvent { event, window_id } => {
                        if self.window.id() == window_id {
                            match event {
                                WindowEvent::Resized(size) => {
                                    self.surface_config.width = size.width;
                                    self.surface_config.height = size.height;

                                    self.surface.configure(&self.device, &self.surface_config);
                                }
                                WindowEvent::KeyboardInput { input, .. } => {
                                    if let Some(key_code) = input.virtual_keycode {
                                        if key_code == VirtualKeyCode::Escape {
                                            running = false;
                                        }

                                        match input.state {
                                            ElementState::Pressed => {
                                                if !pressed_keys.contains(&key_code) {
                                                    pressed_keys.insert(key_code);
                                                }
                                            }
                                            ElementState::Released => {
                                                if pressed_keys.contains(&key_code) {
                                                    pressed_keys.remove(&key_code);
                                                }
                                            }
                                        }
                                    }
                                }
                                WindowEvent::CloseRequested => running = false,
                                _ => {}
                            }
                        }
                    }
                    Event::MainEventsCleared => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Event::DeviceEvent { event, ..} => {
                        if let DeviceEvent::MouseMotion { delta } = event {
                            let rig = &mut self.camera_rig;
                            rig.driver_mut::<YawPitch>().rotate_yaw_pitch(delta.0 as _, -delta.1 as _);
                            rig.update(delta_time);
                        }
                    }

                    _ => {}
                }
            });

            let mut delta_pos = Vec3::ZERO;
            if pressed_keys.contains(&VirtualKeyCode::W) {
                delta_pos += Vec3::new(0.0, 0.0, 1.0);
            }
            if pressed_keys.contains(&VirtualKeyCode::A) {
                delta_pos += Vec3::new(-1.0, 0.0, 0.0);
            }
            if pressed_keys.contains(&VirtualKeyCode::S) {
                delta_pos += Vec3::new(0.0, 0.0, -1.0);
            }
            if pressed_keys.contains(&VirtualKeyCode::D) {
                delta_pos += Vec3::new(1.0, 0.0, 0.0);
            }
            delta_pos = self.camera_rig.final_transform.rotation * delta_pos * 2.0;

            if pressed_keys.contains(&VirtualKeyCode::Space) {
                delta_pos += Vec3::new(0.0, -1.0, 0.0);
            }
            if pressed_keys.contains(&VirtualKeyCode::LShift) {
                delta_pos += Vec3::new(0.0, 1.0, 0.0);
            }

            let camera_rig = &mut self.camera_rig;
            camera_rig.driver_mut::<Position>().translate(-delta_pos * delta_time * 10.0);
            camera_rig.update(delta_time);

            self.render();
        }
    }
}
