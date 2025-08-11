use crate::application::ImageAsset;
use crate::render::texture;
use crate::render::texture::Texture;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue};

pub struct MaterialGpu {
    pub texture_bind_group: BindGroup,
}

pub struct
MaterialManager {
    pub bind_group_layout: BindGroupLayout,
    materials: HashMap<String, MaterialGpu>,
}

impl MaterialManager {
    pub async fn new(device: &Device) -> MaterialManager {
        // let mut material_manager = MaterialManager {
        MaterialManager {
            bind_group_layout: Self::setup_texture_layout(device),
            materials: HashMap::new(),
        }

        // let default_image_asset = ImageAsset {
        //     name: "black".to_string(),
        //     dimensions: TextureDimensions {},
        //     encoding: ImageEncoding::BC1,
        //     data: vec![],
        // };
        // material_manager.load_material(device, queue);
        // TODO load default material
        //
        // material_manager
    }

    pub fn get_material(&self, material_name: &str) -> &MaterialGpu {
        self.materials.get(material_name).unwrap()
    }

    pub fn get_bind_group(&self, material_name: &str) -> &BindGroup {
        &self
            .materials
            .get(material_name)
            .unwrap()
            .texture_bind_group
    }

    pub fn load_material_to_memory(
        &mut self,
        device: &Device,
        queue: &Queue,
        image: ImageAsset,
    ) {
        let name = image.name.clone();
        let diffuse_texture = Texture::from_image(device, queue, image).unwrap();
        let bind_group = Self::build_texture_bind_group(device, &self.bind_group_layout, &diffuse_texture);
        let material = MaterialGpu {
            texture_bind_group: bind_group,
        };
        self.materials.insert(name, material);
    }

    fn build_texture_bind_group(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        diffuse_texture: &texture::Texture,
    ) -> BindGroup {
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        diffuse_bind_group
    }

    fn setup_texture_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
            label: Some("texture_bind_group_layout"),
        })
    }
}
