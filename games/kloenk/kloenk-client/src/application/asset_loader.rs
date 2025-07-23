use crate::application::asset_loader::AssetType::Image;
use hydrox::load_binary;
use image::{DynamicImage, ImageResult};

pub enum AssetType {
    Audio,
    Image(DynamicImage),
    Model,
    Font,
}

pub struct Asset {
    pub asset_type: AssetType,
    pub name: String,
}

pub(crate) struct AssetLoader {}

impl AssetLoader {
    const IMAGE_FILE_NAMES: [&'static str; 6] = ["close_button.webp", "close_button_hover.webp", "grass.webp", "shield.webp", "sword.webp", "tree.webp"];

    pub async fn load_critical_assets() -> Vec<Asset> {
        let mut assets = Vec::new();

        for image_path in Self::IMAGE_FILE_NAMES {
            let data = load_binary(image_path).await.unwrap();
            let decoded_image = Self::decode_image(&data).unwrap();
            let image_asset = Asset {
                asset_type: Image(decoded_image),
                name: image_path.to_string(),
            };
            assets.push(image_asset);
        }

        assets
    }

    // fn decode_image(image_bytes: &[u8]) -> impl Future<Output=ImageResult<DynamicImage>> {
    //     async move {
    //         // let mut decoder = image_webp::WebPDecoder::new(Cursor::new(image_bytes)).unwrap();
    //         // let bytes_per_pixel = if decoder.has_alpha() { 4 } else { 3 };
    //         // let (width, height) = decoder.dimensions();
    //         // let mut data = vec![0; width as usize * height as usize * bytes_per_pixel];
    //         // decoder.read_image(&mut data).unwrap();
    //         // decoder.
    //         image::load_from_memory(image_bytes)
    //     }
    // }

    fn decode_image(image_bytes: &[u8]) -> ImageResult<DynamicImage> {
        image::load_from_memory(image_bytes)
    }
}