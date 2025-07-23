use crate::application::Asset;
use crate::render::model_loader::load_texture;
use crate::render::texture;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue};

pub struct Material {
    pub texture_bind_group: BindGroup,
}

pub struct
MaterialManager {
    pub bind_group_layout: BindGroupLayout,
    materials: HashMap<String, Material>,
}

impl MaterialManager {
    pub async fn new(device: &Device, queue: &Queue, assets: Vec<Asset>) -> MaterialManager {
        let mut material_manager = MaterialManager {
            bind_group_layout: Self::setup_texture_layout(device),
            materials: HashMap::new(),
        };
        material_manager.load_materials(device, queue, assets).await;
        material_manager
    }

    pub fn get_material(&self, material_name: &str) -> &Material {
        self.materials.get(material_name).unwrap()
    }

    pub fn get_bind_group(&self, material_name: &str) -> &BindGroup {
        &self
            .materials
            .get(material_name)
            .unwrap()
            .texture_bind_group
    }

    async fn load_materials(&mut self, device: &Device, queue: &Queue, assets: Vec<Asset>) {
        let materials = &mut self.materials;
        let layout = &self.bind_group_layout;
        for asset in assets {
            // let Image(image) = asset.asset_type;
            let asset_name = asset.name.clone();
            let material = Self::load_material(device, queue, layout, asset).await.unwrap();

            materials.insert(
                asset_name,
                material,
            );
        }

        //
        // materials.insert(
        //     "shield".to_string(),
        //     Self::load_material(device, queue, layout, "shield.webp")
        //         .await
        //         .unwrap(),
        // );
        // materials.insert(
        //     "grass".to_string(),
        //     Self::load_material(device, queue, layout, "grass.webp")
        //         .await
        //         .unwrap(),
        // );
        // materials.insert(
        //     "tree".to_string(),
        //     Self::load_material(device, queue, layout, "tree.webp")
        //         .await
        //         .unwrap(),
        // );
        //
        // materials.insert(
        //     "close_button".to_string(),
        //     Self::load_material(device, queue, layout, "close_button.webp")
        //         .await
        //         .unwrap(),
        // );
        //
        // materials.insert(
        //     "close_button_hover".to_string(),
        //     Self::load_material(device, queue, layout, "close_button_hover.webp")
        //         .await
        //         .unwrap(),
        // );
    }

    async fn load_material(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        image: Asset,
    ) -> anyhow::Result<Material> {
        let diffuse_texture = load_texture(device, queue, image).await?;
        let bind_group = Self::build_texture_bind_group(device, layout, &diffuse_texture);
        Ok(Material {
            texture_bind_group: bind_group,
        })
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
