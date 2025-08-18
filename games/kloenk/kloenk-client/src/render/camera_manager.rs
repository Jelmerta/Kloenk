use crate::render::camera::Camera;
use cgmath::SquareMatrix;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};

pub struct CameraManager {
    pub bind_group_layout: BindGroupLayout,
    camera_contexts: HashMap<String, CameraContext>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_projection(&mut self, camera: &mut Camera) {
        self.view_projection = camera.view_projection_matrix.into();
    }
}

pub struct CameraContext {
    uniform: CameraUniform,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl CameraManager {
    pub fn new(device: &Device) -> Self {
        let mut manager = Self {
            bind_group_layout: Self::setup_bind_group_layouts(device),
            camera_contexts: HashMap::new(),
        };
        manager.build_camera_contexts(device);
        manager
    }

    pub fn update_buffer(&mut self, camera_name: &str, queue: &Queue, camera: &mut Camera) {
        let context = self
            .camera_contexts
            .get_mut(camera_name)
            .expect("Camera contexts should exist");

        context.uniform.update_view_projection(camera);

        queue.write_buffer(&context.buffer, 0, bytemuck::cast_slice(&[context.uniform]));
    }

    pub fn get_bind_group(&self, camera_name: &str) -> &BindGroup {
        &self
            .camera_contexts
            .get(camera_name)
            .expect("Camera contexts should exist")
            .bind_group
    }

    fn build_camera_contexts(&mut self, device: &Device) {
        let camera_context_3d = self.build_camera_context(device);
        self.camera_contexts
            .insert("camera_3d".to_owned(), camera_context_3d);
        let camera_context_2d = self.build_camera_context(device);
        self.camera_contexts
            .insert("camera_2d".to_owned(), camera_context_2d);
    }

    fn build_camera_context(&mut self, device: &Device) -> CameraContext {
        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        CameraContext {
            uniform: camera_uniform,
            buffer: camera_buffer,
            bind_group: camera_bind_group,
        }
    }

    fn setup_bind_group_layouts(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}
