use crate::application::ImageAsset;
use crate::render::model::TextureDefinition;
use crate::render::texture::Texture;
use std::collections::HashMap;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue};

pub struct TextureManager {
    pub bind_group_layout: BindGroupLayout,
    textures_gpu: HashMap<String, BindGroup>,
}

impl TextureManager {
    pub fn new(device: &Device, queue: &Queue) -> TextureManager {
        let texture_layout = Self::setup_texture_layout(device);
        let white_texture = Texture::white_1x1(device, queue).unwrap();

        let white_texture_bind_group =
            Self::build_texture_bind_group(device, &texture_layout, &white_texture);
        let mut textures = HashMap::new();
        textures.insert("white".to_owned(), white_texture_bind_group);

        TextureManager {
            bind_group_layout: texture_layout,
            textures_gpu: textures,
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

    // pub fn get_texture(&self, material_name: &str) -> &TexturesGpu {
    //     self.textures.get(material_name).unwrap()
    // }

    // TODO if none found, 1x1 white texture
    // pub fn get_bind_group(&self, material_name: &str) -> &BindGroup {
    pub fn get_bind_group(&self, texture_definition: Option<&TextureDefinition>) -> &BindGroup {
        let texture_id = texture_definition.as_ref().map(|td| td.id.as_str());
        self
            .textures_gpu
            .get(texture_id.unwrap_or("white"))
            .unwrap()
    }

    pub fn load_material_to_memory(&mut self, device: &Device, queue: &Queue, image: &ImageAsset) {
        let name = image.name.clone();
        let diffuse_texture = Texture::from_image(device, queue, image).unwrap();
        let bind_group =
            Self::build_texture_bind_group(device, &self.bind_group_layout, &diffuse_texture);
        self.textures_gpu.insert(name, bind_group);
    }

    fn build_texture_bind_group(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        diffuse_texture: &Texture,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        })
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
