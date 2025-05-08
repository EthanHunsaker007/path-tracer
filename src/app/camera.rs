use cgmath::{Angle, InnerSpace};
use wgpu::util::DeviceExt;

pub struct PhysicalCamera {
    pub forward: cgmath::Vector3<f32>,
    pub right: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub pitch: cgmath::Deg<f32>,
    pub yaw: cgmath::Deg<f32>,
    pub position: cgmath::Point3<f32>,
    aspect: f32,
    viewport_height: f32,
    sensor_pixel_size: cgmath::Vector2<f32>,
}

impl PhysicalCamera {
    fn get_axes(
        &self,
    ) -> (
        cgmath::Vector3<f32>,
        cgmath::Vector3<f32>,
        cgmath::Vector3<f32>,
    ) {
        let pitch_rad: cgmath::Rad<f32> = self.pitch.into();
        let yaw_rad: cgmath::Rad<f32> = self.yaw.into();

        let forward = cgmath::vec3(
            yaw_rad.cos() * pitch_rad.cos(),
            -pitch_rad.sin(),
            -yaw_rad.sin() * pitch_rad.cos(),
        );
        let right = forward.cross(cgmath::vec3(0.0, -1.0, 0.0)).normalize();
        let up = right.cross(forward).normalize();

        (forward, right, up)
    }

    fn set_axes(&mut self) {
        (self.forward, self.right, self.up) = self.get_axes();
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.sensor_pixel_size = (size.width as f32, size.height as f32).into();
        self.aspect = self.sensor_pixel_size.x / self.sensor_pixel_size.y;
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

struct CameraUniform {
    position: [f32; 3],
    _pad0: f32,
    lower_left_pixel: [f32; 3],
    _pad1: f32,
    pixel_delta_x: [f32; 3],
    _pad2: f32,
    pixel_delta_y: [f32; 3],
    _pad3: f32,
}

pub struct Camera {
    pub camera: PhysicalCamera,
    uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(size: &winit::dpi::PhysicalSize<u32>, device: &wgpu::Device, queue: &wgpu::Queue, fov: f32) -> Camera {

        let camera = PhysicalCamera {
            forward: (0.0, 0.0, -1.0).into(),
            right: (-1.0, 0.0, 0.0).into(),
            up: (0.0, -1.0, 0.0).into(),
            pitch: cgmath::Deg(0.0),
            yaw: cgmath::Deg(90.0),
            position: (0.0, 0.0, 0.0).into(),
            aspect: size.width as f32 / size.height as f32,
            viewport_height: 2.0 / f32::tan(fov / 2.0),
            sensor_pixel_size: cgmath::vec2(size.width as f32, size.height as f32)
        };

        let uniform = CameraUniform::default();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut camera_struct = Camera {
            camera,
            uniform,
            buffer,
        };

        camera_struct.build_uniform();
        camera_struct.update_buffer(queue);

        camera_struct
    }

    pub fn set_position(&mut self, position: cgmath::Point3<f32>, queue: &wgpu::Queue) {
        self.camera.position = position;
        self.build_uniform();
        self.update_buffer(&queue);
    }

    pub fn set_rotation(&mut self, pitch: f32, yaw: f32, queue: &wgpu::Queue) {
        self.camera.yaw.0 = yaw;
        self.camera.pitch.0 = pitch.clamp(-89.99, 89.99);
        self.camera.set_axes();
        self.build_uniform();
        self.update_buffer(&queue);
    }

    pub fn set_fov(& mut self, fov: f32, queue: &wgpu::Queue) {
        self.camera.viewport_height = 2.0 / f32::tan(fov / 2.0);
        self.build_uniform();
        self.update_buffer(&queue);

    }
    
    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );     
    }

    pub fn build_uniform(&mut self) {
        let viewport_width = self.camera.viewport_height * self.camera.aspect;

        let viewport_u = self.camera.right * viewport_width;
        let viewport_v = self.camera.up * -self.camera.viewport_height;

        let pixel_delta_x = viewport_u / self.camera.sensor_pixel_size.x;
        let pixel_delta_y = viewport_v / self.camera.sensor_pixel_size.y;

        let lower_left = self.camera.position + self.camera.forward * 1.0 - viewport_u * 0.5 - viewport_v * 0.5;
        let lower_left_pixel: [f32; 3] =
            (lower_left + 0.5 * (pixel_delta_x - pixel_delta_y)).into();


        self.uniform = CameraUniform {
            position: self.camera.position.into(),
            _pad0: 0.0,
            lower_left_pixel,
            _pad1: 0.0,
            pixel_delta_x: pixel_delta_x.into(),
            _pad2: 0.0,
            pixel_delta_y: pixel_delta_y.into(),
            _pad3: 0.0,
        }
    }
}