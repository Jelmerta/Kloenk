use crate::application::{ImageAsset, ImageEncoding};
use anyhow::{Ok, Result};
use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

const BC_BLOCK_SIZE: u32 = 4;

pub struct Texture {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub struct Depth {
    pub view: wgpu::TextureView,
}

impl Texture {
    // Reference wrapper method, we perform a lot of similar actions: device.create_texture_with_data(queue, desc, order, data);
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &ImageAsset,
    ) -> Result<Self> {
        let texture_format = match image.encoding {
            ImageEncoding::BC1 => TextureFormat::Bc1RgbaUnormSrgb,
            ImageEncoding::BC7 => TextureFormat::Bc7RgbaUnormSrgb,
        };

        let blocks_wide = image.dimensions.pixel_width.div_ceil(BC_BLOCK_SIZE);
        let blocks_high = image.dimensions.pixel_height.div_ceil(BC_BLOCK_SIZE);

        let padded_width = blocks_wide * BC_BLOCK_SIZE;
        let padded_height = blocks_high * BC_BLOCK_SIZE;

        let texture_size = Extent3d {
            width: padded_width,
            height: padded_height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some(&image.name),
            view_formats: &[],
        });

        let bytes_per_block = match image.encoding {
            ImageEncoding::BC1 => 8,
            ImageEncoding::BC7 => 16,
        };

        let bytes_per_row = blocks_wide * bytes_per_block;


        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(image.dimensions.pixel_height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Image view"),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { view, sampler })
    }

    pub fn white_1x1(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("white 1x1"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            size,
        );

        let view = white_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Image view white"),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { view, sampler })
    }
}

impl Depth {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let descriptor = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = device.create_texture(&descriptor);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Depth view"),
            ..Default::default()
        });
        Self { view }
    }
}
