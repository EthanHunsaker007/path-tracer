pub struct BindGroups {
    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
    pub fragment_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_buffer_bind_group_layout: wgpu::BindGroupLayout,
    pub scene_bind_group: wgpu::BindGroup,
    pub camera_bind_group: wgpu::BindGroup,
    pub compute_bind_group: wgpu::BindGroup,
    pub fragment_bind_group: wgpu::BindGroup,
    pub bind_group_a_read_b_write: wgpu::BindGroup,
    pub bind_group_b_read_a_write: wgpu::BindGroup,
}

impl BindGroups {
    pub fn new(
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        sphere_buffer: &wgpu::Buffer,
        material_buffer: &wgpu::Buffer,
        camera_buffer: &wgpu::Buffer,
        frame_buffer: &Option<wgpu::Buffer>,
        texture_buffer_a: &wgpu::Buffer,
        texture_buffer_b: &wgpu::Buffer,
        texture_view: &wgpu::TextureView,
    ) -> BindGroups {
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scene Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Bind Group"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sphere_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: material_buffer.as_entire_binding(),
            }],
        });
        
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
            },
            
            ],
        });

        let frame_buf = frame_buffer
        .as_ref()
        .expect("fuck"); 

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, 
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None 
                        },
                        count: None,
                    }
                ],
            });
        
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: frame_buf.as_entire_binding(),
                }
            ],
        });

        let texture_buffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Buffer Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            });

        let bind_group_a_read_b_write = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: texture_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: texture_buffer_b.as_entire_binding(),
                },
            ],
            label: Some("BindGroup A->B"),
        });
        
        let bind_group_b_read_a_write = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: texture_buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: texture_buffer_a.as_entire_binding(),
                },
            ],
            label: Some("BindGroup B->A"),
        });
            
        let fragment_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let fragment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment Bind Group"),
            layout: &fragment_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        BindGroups {
            scene_bind_group_layout,
            camera_bind_group_layout,
            compute_bind_group_layout,
            fragment_bind_group_layout,
            texture_buffer_bind_group_layout,
            scene_bind_group,
            camera_bind_group,
            compute_bind_group,
            fragment_bind_group,
            bind_group_a_read_b_write,
            bind_group_b_read_a_write,
        }
    }

    pub fn rebuild_fragment_bind_group(
        &mut self,         
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        texture_view: &wgpu::TextureView,
    ) {
        self.fragment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment Bind Group"),
            layout: &self.fragment_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
    }

    pub fn rebuild_compute_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame_buffer: &Option<wgpu::Buffer>,
        texture_view: &wgpu::TextureView,
    ) {
        let frame_buf = frame_buffer
        .as_ref()
        .expect("fuck"); 

        self.compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &self.compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: frame_buf.as_entire_binding(),
                }
            ],
        });
    }

    pub fn rebuild_texture_buffer_bind_groups(
        &mut self,
        device: &wgpu::Device,
        texture_buffer_a: &wgpu::Buffer,
        texture_buffer_b: &wgpu::Buffer,

    ) {
        self.bind_group_a_read_b_write = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: texture_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: texture_buffer_b.as_entire_binding(),
                },
            ],
            label: Some("BindGroup A->B"),
        });
        
        self.bind_group_b_read_a_write = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: texture_buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: texture_buffer_a.as_entire_binding(),
                },
            ],
            label: Some("BindGroup B->A"),
        });
    }

    pub fn rebuild_scene_bind_group(
        &mut self,
        device: &wgpu::Device,
        sphere_buffer: &wgpu::Buffer,
        material_buffer: &wgpu::Buffer,

    ) {
        self.scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Bind Group"),
            layout: &self.scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sphere_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: material_buffer.as_entire_binding(),
            }],
        });
    }
}