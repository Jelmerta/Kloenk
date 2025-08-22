use crate::application::{ImageAsset, ImageEncoding};
use anyhow::{Ok, Result};
use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

pub struct Texture {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

// Probably just a specific instance of Texture?
pub struct Depth {
    pub view: wgpu::TextureView,
}

impl Texture {
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &ImageAsset,
    ) -> Result<Self> {
        let texture_format = match image.encoding {
            ImageEncoding::BC1 => TextureFormat::Bc1RgbaUnormSrgb,
            ImageEncoding::BC7 => TextureFormat::Bc7RgbaUnormSrgb,
        };

        let block_size = 4; // TODO static constant thing
        let blocks_wide = image.dimensions.pixel_width.div_ceil(block_size);
        let blocks_high = image.dimensions.pixel_height.div_ceil(block_size);

        let texture_size = Extent3d {
            width: blocks_wide * block_size,  // padded width
            height: blocks_high * block_size, // padded height
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
        // Based on blocks, not pixels
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
                // bytes_per_row: Some(4 * image.dimensions.pixel_width),  rgba
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(image.dimensions.pixel_height),
            },
            texture_size,
        );

        // TODO difference?
        // device.create_texture_with_data()

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Image view"),
            // format: Some(self.config.format.add_srgb_suffix()), ?
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

    // Or generally for rgb
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
            usage: TextureUsages::RENDER_ATTACHMENT, // | TextureUsages::TEXTURE_BINDING, TODO depth is not used in shader is it? dont need texture binding? unlike what sotrh suggests?
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
