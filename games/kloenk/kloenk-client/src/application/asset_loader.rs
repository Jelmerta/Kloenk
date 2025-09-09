use ddsfile::{Dds, DxgiFormat, FourCC, Header, Header10};
use hydrox::load_binary;

#[derive(Debug)]
pub struct ImageAsset {
    pub name: String,
    pub dimensions: TextureDimensions,
    pub encoding: ImageEncoding,
    pub data: Vec<u8>,
}

pub struct FontAsset {
    #[allow(dead_code)]
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ImageEncoding {
    BC1,
    BC7,
}

#[derive(Debug)]
pub struct TextureDimensions {
    pub pixel_width: u32,
    pub pixel_height: u32,
}

pub struct AssetLoader {}

impl AssetLoader {
    pub async fn load_image_asset(image_path: &str) -> ImageAsset {
        let data = load_binary(image_path).await.unwrap(); // Maybe retry? How can this fail?
        Self::load_dds(image_path, &data)
    }

    fn load_dds(image_name: &str, dds_bytes: &[u8]) -> ImageAsset {
        let dds = Dds::read(dds_bytes).unwrap(); // Maybe retry? How can this fail? Bytes are already in memory...
        let format = detect_format(&dds.header, dds.header10.as_ref());
        ImageAsset {
            name: image_name.to_owned(),
            dimensions: TextureDimensions {
                pixel_width: dds.header.width,
                pixel_height: dds.header.height,
            },
            encoding: format,
            data: dds.data,
        }
    }
}

fn detect_format(header: &Header, header10: Option<&Header10>) -> ImageEncoding {
    if let Some(header10) = header10 {
        match header10.dxgi_format {
            DxgiFormat::BC7_UNorm => ImageEncoding::BC7,
            DxgiFormat::BC1_UNorm => ImageEncoding::BC1,
            _ => panic!("Unexpected DXGI format {:?}", header10.dxgi_format),
        }
    } else {
        match header.spf.fourcc {
            Some(FourCC(FourCC::BC1_UNORM)) => ImageEncoding::BC1,
            _ => panic!("Unexpected DXGI format {:?}", header.spf.fourcc),
        }
    }
}
