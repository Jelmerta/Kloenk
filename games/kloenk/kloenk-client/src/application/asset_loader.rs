use ddsfile::{Dds, DxgiFormat, FourCC, Header, Header10};
use hydrox::load_binary;

pub enum AssetType {
    // Audio,
    Image(ImageAsset),
    // Model,
    // Font,
}

pub struct Asset {
    pub asset_type: AssetType,
    // pub name: String,
}

#[derive(Debug)]
pub struct ImageAsset {
    pub name: String,
    pub dimensions: TextureDimensions,
    pub encoding: ImageEncoding,
    // pub has_alpha: bool,
    pub data: Vec<u8>,
}

// TODO should probably contain transcoded image?
#[derive(Debug)]
pub enum ImageEncoding {
    BC1,
    // BC1A,
    BC7,
}

// impl PartialEq for ImageEncoding {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (ImageEncoding::BasisLz, ImageEncoding::BasisLz) => true,
//             (ImageEncoding::Uastc, ImageEncoding::Uastc) => true,
//             _ => false,
//         }
//     }
// }

#[derive(Debug)]
pub struct TextureDimensions {
    pub pixel_width: u32,
    pub pixel_height: u32,
}

pub struct AssetLoader {
    // primitive_vertices_manager: PrimitiveVerticesManager,
    // material_manager: MaterialManager,
}

impl AssetLoader {
    // pub(crate) const STARTUP_GPU_ASSETS: [&'static str; 6] = [
    //     "close_button.dds",
    //     "close_button_hover.dds",
    //     "grass.dds",
    //     "shield.dds",
    //     "sword.dds",
    //     "tree.dds",
    // ];
    // gozer.gltf?

    // pub fn new(primitive_vertices_manager: PrimitiveVerticesManager, material_manager: MaterialManager) -> Self {
    //     AssetLoader {
    //         primitive_vertices_manager,
    //         material_manager,
    //     }
    // }

    pub async fn load_image_asset(image_path: &str) -> ImageAsset {
        let data = load_binary(image_path).await.unwrap();
        Self::load_dds(image_path, &data)
    }

    fn load_dds(image_name: &str, dds_bytes: &[u8]) -> ImageAsset {
        let dds = Dds::read(dds_bytes).unwrap();
        let format = detect_format(&dds.header, &dds.header10);
        ImageAsset {
            name: image_name.to_string(),
            dimensions: TextureDimensions { pixel_width: dds.header.width, pixel_height: dds.header.height },
            encoding: format,
            // has_alpha: false,
            data: dds.data,
        }
    }
}

fn detect_format(header: &Header, header10: &Option<Header10>) -> ImageEncoding {
    if let Some(header10) = header10 {
        match header10.dxgi_format {
            DxgiFormat::BC7_UNorm => ImageEncoding::BC7,
            DxgiFormat::BC1_UNorm => ImageEncoding::BC1,
            _ => panic!("Unexpected DXGI format {:?}", header10.dxgi_format),
        }
    } else {
        match header.spf.fourcc.clone().unwrap() {
            FourCC(FourCC::BC1_UNORM) => ImageEncoding::BC1,
            _ => panic!("Unexpected DXGI format {:?}", header.spf.fourcc.clone().unwrap()),
        }
    }
}
