use rand::prelude::*;
use std::vec;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pos_and_rad: [f32; 4],
    material: [u32; 4],
}

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
    pub spheres: Vec<Sphere>,
    pub sphere_buffer: wgpu::Buffer,
    pub material_buffer: wgpu::Buffer,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Scene {
        let materials: Vec<Material> = vec![Material::default()];
        let spheres: Vec<Sphere> = vec![Sphere::default()];

        let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Buffer"),
            contents: bytemuck::cast_slice(&spheres),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Buffer"),
            contents: bytemuck::cast_slice(&materials),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Scene {
            materials,
            spheres,
            sphere_buffer,
            material_buffer,
        }
    }

    pub fn add_sphere(&mut self, radius: f32, pos: [f32; 3], material: u32) {
        self.spheres.push(Sphere {
            pos_and_rad: [pos[0], pos[1], pos[2], radius],
            material: [material, 0, 0, 0],
        });
    }

    pub fn setup_test_scene(&mut self, sphere_count: u32, device: &wgpu::Device) {
        self.spheres.clear();
        self.materials.clear();
        self.materials
            .push(Material::new([0.5; 3], [0.0; 3], 0.0, 0.0, 0.0));

        for _i in 1..sphere_count {
            let emission: [f32; 3] = if rand::rng().random::<f32>() > 0.8 {
                [
                    rand::rng().random::<f32>().powf(1.0 / 2.0),
                    rand::rng().random::<f32>().powf(1.0 / 2.0),
                    rand::rng().random::<f32>().powf(1.0 / 2.0),
                ]
            } else {
                [0.0; 3]
            };
            let rough_rand = rand::rng().random::<f32>();
            let roughness = if rough_rand > 0.9 {
                0.0
            } else if rough_rand > 0.2 {
                1.0
            } else {
                rand::rng().random::<f32>()
            };
            self.materials.push(Material::new(
                [
                    rand::rng().random::<f32>(),
                    rand::rng().random::<f32>(),
                    rand::rng().random::<f32>(),
                ],
                emission,
                rand::rng().random_range(0..3) as f32,
                roughness,
                1.5,
            ));
        }

        for i in 1..sphere_count {
            self.add_sphere(
                rand::rng().random::<f32>() * 5.0,
                [
                    (rand::rng().random::<f32>() - 0.5) * 100.0,
                    rand::rng().random::<f32>() * -100.0,
                    rand::rng().random::<f32>() * 100.0,
                ],
                i,
            );

        }
        self.add_sphere(2000.0, [0.0, 2013.0, 0.0], 0);
        self.update_material_buffer(device);
        self.update_sphere_buffer(device);
    }

    pub fn update_sphere_buffer(&mut self, device: &wgpu::Device) {
        self.sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Buffer"),
            contents: bytemuck::cast_slice(&self.spheres),
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
