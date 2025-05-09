//use rand::prelude::*;
use std::vec;
use rand::Rng;
use wgpu::util::DeviceExt;
use crate::mesh::*;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    albedo_and_mat: [f32; 4],
    emission_and_roughness: [f32; 4],
    ior: [f32; 4],
}

impl Material {
    pub fn new(albedo: [f32; 3], emission: [f32; 3], material: f32, roughness: f32, ior: f32) -> Self {
        Material {
            albedo_and_mat: [albedo[0], albedo[1], albedo[2], material],
            emission_and_roughness: [emission[0], emission[1], emission[2], roughness],
            ior: [ior, 0.0, 0.0, 0.0],
        }
    }
}

pub struct Scene {
    pub materials: Vec<Material>,
    pub vertices: Vec<[f32; 4]>,
    pub tris: Vec<[u32; 4]>,
    pub material_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub tri_buffer: wgpu::Buffer,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Scene {
        let materials = vec![Material::default()];
        let vertices = vec![[0.0; 4]];
        let tris = vec![[0; 4]];

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer"),
            contents: bytemuck::cast_slice(&materials),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let tri_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Buffer"),
            contents: bytemuck::cast_slice(&tris),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Scene {
            materials,
            vertices,
            tris,
            material_buffer,
            vertex_buffer,
            tri_buffer
        }
    }

    pub fn setup_test_scene(&mut self, device: &wgpu::Device) {
        self.materials.clear();
        self.vertices.clear();
        self.tris.clear();

        self.materials
            .push(Material::new([rand::rng().random::<f32>(), rand::rng().random::<f32>(), rand::rng().random::<f32>()], [0.0; 3], 2.0, 0.5, 1.5));

        (self.vertices, self.tris) = parse_obj("models/apple.obj");

        self.update_material_buffer(device);
        self.update_triangle_buffers(device);
    }

    pub fn update_triangle_buffers(&mut self, device: &wgpu::Device) {
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.tri_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Buffer"),
            contents: bytemuck::cast_slice(&self.tris),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
    }

    pub fn update_material_buffer(&mut self, device: &wgpu::Device) {
        self.material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer"),
            contents: bytemuck::cast_slice(&self.materials),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
    }
}
