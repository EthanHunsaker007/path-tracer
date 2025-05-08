use std::vec;
use wgpu::util::DeviceExt;

pub struct Textures {
    pub texture_buffer_a: wgpu::Buffer,
    pub texture_buffer_b: wgpu::Buffer,
    pub surface_texture_view: wgpu::TextureView,
}

impl Textures {
    pub fn new(device: &wgpu::Device, size: &winit::dpi::PhysicalSize<u32>) -> Textures {
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let blank_buffer: Vec<[f32; 4]> = vec![[0.0; 4]; (size.width * size.height) as usize];

        let surface_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Diffuse Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let texture_buffer_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Buffer A"),
            contents: bytemuck::cast_slice(&blank_buffer),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let texture_buffer_b = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Buffer B"),
            contents: bytemuck::cast_slice(&blank_buffer),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let surface_texture_view =
            surface_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Textures {
            texture_buffer_a,
            texture_buffer_b,
            surface_texture_view,
        }
    }
}