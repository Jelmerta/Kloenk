use crate::render::model::ColorDefinition;
use cgmath::Vector4;
use std::collections::HashMap;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BindGroupLayout, BindingType, Device};

pub struct ColorGpu {
    pub bind_group: BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorUniform {
    primitive_color: [f32; 4],
}

pub struct ColorManager {
    pub bind_group_layout: BindGroupLayout,
    colors: HashMap<String, ColorGpu>,
}

impl ColorManager {
    pub fn new(device: &Device) -> Self {
        let mut color_manager = ColorManager {
            bind_group_layout: Self::setup_color_layout(device),
            colors: HashMap::new(),
        };
        let white_color_definition = ColorDefinition {
            id: "white".to_string(),
            value: Vector4::new(1.0, 1.0, 1.0, 1.0),
        };
        color_manager.load_color_to_memory(device, white_color_definition);
        color_manager
    }

    pub fn get_color_bind_group(&self, color: &str) -> &BindGroup {
        &self.colors.get(color).unwrap().bind_group
    }

    pub fn load_color_to_memory(&mut self, device: &Device, color_definition: ColorDefinition) {
        let color_name = color_definition.id.clone();
        let color = ColorGpu {
            bind_group: Self::build_color_bind_group(
                device,
                &self.bind_group_layout,
                color_definition.value,
            ),
        };
        self.colors.insert(color_name, color);
    }

    fn build_color_bind_group(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        color_to_load: Vector4<f32>,
    ) -> BindGroup {
        let color_uniform = ColorUniform {
            primitive_color: color_to_load.into(),
        };
        let color_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Color Buffer"),
            contents: bytemuck::cast_slice(&[color_uniform]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Bind Group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
        })
    }

    fn setup_color_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}
