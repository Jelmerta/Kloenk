use crate::application::AssetType::Image;
use crate::application::{Asset, AssetType};
use anyhow::{Ok, Result};
use image::{DynamicImage, GenericImageView, ImageResult};
// use image::{DynamicImage, GenericImageView, ImageResult};

pub struct Texture {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

// Probably just a specific instance of Texture?
pub struct Depth {
    pub view: wgpu::TextureView,
}

impl Texture {
    pub async fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // image_bytes: &[u8],
        image: Asset,
        // label: &str,
    ) -> Result<Self> {
        // Decoding takes a bit of time. We do not want to block on this task. TODO does not actually seem to help anything...? Probably because there's not really any other tasks left to do, image loading tasks take the longest amount of time. Could have placeholder until it is decoded
        // let result = web_sys::window().unwrap().create_image_bitmap_with_blob(bytes);
        // let diffuse_image = Self::decode_image(bytes).await?;
        Self::from_image(device, queue, image)
        // Self::from_image(device, queue, &image, Some(label))
    }

    fn decode_image(image_bytes: &[u8]) -> impl Future<Output=ImageResult<DynamicImage>> {
        async move {
            // let mut decoder = image_webp::WebPDecoder::new(Cursor::new(image_bytes)).unwrap();
            // let bytes_per_pixel = if decoder.has_alpha() { 4 } else { 3 };
            // let (width, height) = decoder.dimensions();
            // let mut data = vec![0; width as usize * height as usize * bytes_per_pixel];
            // decoder.read_image(&mut data).unwrap();
            // decoder.
            image::load_from_memory(image_bytes)
        }
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: Asset,
        // label: Option<&str>,
    ) -> Result<Self> {
        let dynamic_image;
        match image.asset_type {
            AssetType::Audio => { panic!("unexpected asset type"); }
            Image(img) => { dynamic_image = img; }
            AssetType::Model => { panic!("unexpected asset type"); }
            AssetType::Font => { panic!("unexpected asset type"); }
        } //= image.asset_type;
        let diffuse_rgba = dynamic_image.to_rgba8();

        let dimensions = dynamic_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            // label,
            label: Some(&image.name),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

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
}

impl Depth {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
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
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[Self::DEPTH_FORMAT],
        };
        let texture = device.create_texture(&descriptor);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Depth view"),
            ..Default::default()
        });
        Self { view }
    }
}
