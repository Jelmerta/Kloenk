use crate::application::asset_loader::AssetType::Image;
use basis_universal::sys::{low_level_uastc_transcoder_new, low_level_uastc_transcoder_transcode_slice, LowLevelUastcTranscoder};
use basis_universal::{
    BasisTextureFormat, Compressor, CompressorParams, TranscodeParameters, Transcoder,
    TranscoderTextureFormat, UserData,
};
use cgmath::num_traits::ToPrimitive;
use hydrox::load_binary;
use itertools::Itertools;
use ktx2::{ColorModel, DfdBlockBasic, DfdBlockHeaderBasic, DfdHeader, Format, SampleInformation, SupercompressionScheme};
use std::ptr::read;
use zstd::bulk::decompress;
// Add zstd crate to dependencies

pub enum AssetType {
    Audio,
    Image(ImageAsset),
    Model,
    Font,
}

pub struct Asset {
    pub asset_type: AssetType,
    pub name: String,
}

#[derive(Debug)]
pub struct ImageAsset {
    pub name: String,
    pub dimensions: TextureDimensions,
    pub encoding: ImageEncoding,
    pub has_alpha: bool,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ImageEncoding {
    BasisLz,
    Uastc,
}

#[derive(Debug)]
pub struct TextureDimensions {
    pub pixel_width: u32,
    pub pixel_height: u32,
}

pub struct AssetLoader {}

impl AssetLoader {
    pub(crate) const STARTUP_GPU_ASSETS: [&'static str; 6] = [
        "close_button.ktx2",
        "close_button_hover.ktx2",
        "grass.ktx2",
        "shield.ktx2",
        "sword.ktx2",
        "tree.ktx2",
    ];

    pub async fn load_critical_assets() -> Vec<Asset> {
        let mut assets = Vec::new();

        for image_path in Self::STARTUP_GPU_ASSETS {
            let image_asset = Self::load_image_asset(image_path).await;
            let asset = Asset {
                asset_type: Image(image_asset),
                name: image_path.to_string(),
            };
            assets.push(asset);
        }

        assets
    }

    pub async fn load_image_asset(image_path: &str) -> ImageAsset {
        let data = load_binary(image_path).await.unwrap();
        Self::load_image(image_path, &data)
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

    // fn decode_image(image_bytes: &[u8]) -> ImageResult<DynamicImage> {
    //     image::load_from_memory(image_bytes)
    // }

    // Helpful: https://github.com/woelper/oculante/blob/680faabd105435b7c2668bcd3be715e28aa9605e/src/ktx2_loader/ktx2.rs
    fn load_image(image_name: &str, image_bytes: &[u8]) -> ImageAsset {
        log::error!("Loading image: {}", image_name);
        let reader = ktx2::Reader::new(image_bytes).unwrap();

        // Handle supercompression TODO handle etc1s?
        let data;
        let supercompress_scheme = reader.header().supercompression_scheme;
        if reader.header().supercompression_scheme.is_some() {
            let scheme = supercompress_scheme.unwrap();
            if let SupercompressionScheme::Zstandard = scheme {
                let level = reader.levels().next().unwrap(); // Assume 1 level
                //                     let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut cursor)?
                data = decompress(level.data, level.uncompressed_byte_length.to_usize().unwrap()).unwrap();
            } else {
                panic!("Unsupported supercompression scheme: {:?}", supercompress_scheme);
            }
        } else {
            panic!("Unsupported supercompression scheme: {:?}", supercompress_scheme);
        }

        // Load GPU format. Note: Header format is empty when supercompression is used
        log::error!("Loading format: {:?}", reader.header());
        let pixel_width = reader.header().pixel_width;
        let pixel_height = reader.header().pixel_height;
        let dfd_block = reader.dfd_blocks().next().unwrap(); // TODO are there many? could some have other DFD model? basically only need to check one value which should be the same for the whole image?

        // dfd_block.header
        // DfdHeader::BASIC
        // DfdBlockHeaderBasic
        // let x :DfdBlockBasic
        // let block = DfdBlockBasic::parse(dfd_block.data.try_into().unwrap()).unwrap();
        let block_basic = DfdBlockBasic::parse(dfd_block.data).unwrap(); // TODO assume header is contained in data?
        let color_model = block_basic.header.color_model.unwrap();
        let encoding = match color_model {
            ColorModel::UASTC => ImageEncoding::Uastc,
            ColorModel::ETC1S => ImageEncoding::BasisLz,
            _ => {
                panic!("Unsupported color model: {:?}", color_model);
            }
        };
        log::error!("Encoding: {:?}", encoding);


        //  Some(ColorModel::UASTC) => {
        //             return Err(TextureError::FormatRequiresTranscodingError(
        //                 TranscodeFormat::Uastc(match sample_information[0].channel_type {
        //                     0 => DataFormat::Rgb,
        //                     3 => DataFormat::Rgba,
        //                     4 => DataFormat::Rrr,
        //                     5 => DataFormat::Rrrg,
        //                     6 => DataFormat::Rg,
        //                     channel_type => {
        //                         return Err(TextureError::UnsupportedTextureFormat(format!(
        //                             "Invalid KTX2 UASTC channel type: {channel_type}",
        //                         )))
        //                     }
        //                 }),
        //             ));
        //         }

        // TODO they dont transcode this? you can do that i believe hm
        // // ETC1 a subset of ETC2 only supporting Rgb8
        // Some(ColorModel::ETC1) => {
        //     if is_srgb {
        //         TextureFormat::Etc2Rgb8UnormSrgb
        //     } else {
        //         TextureFormat::Etc2Rgb8Unorm
        //     }
        // }


        let sample_information = block_basic.sample_information().next().unwrap();
        let has_alpha = match sample_information.channel_type {
            3 => true,
            0 => false,
            _ => panic!(
                "Unsupported channel type {:?}",
                sample_information.channel_type
            ),
        };
        log::error!("alpha: {}", has_alpha);

        // Transcoding UASTC
        let transcoder = basis_universal::LowLevelUastcTranscoder::new();
        transcoder.transcode_slice(data, )
        // let transcoder = LowLevelUastcTranscoder::
        // let transcoder = low_level_uastc_transcoder_new;
        low_level_uastc_transcoder_transcode_slice(transcoder);
        LowLevelUastcTranscoder::low_level_uastc_transcoder_new();


        // let encoding = match supercompression_scheme {
        //     Some(SupercompressionScheme::BasisLZ) => ImageEncoding::BasisLz,
        //     None => ImageEncoding::Uastc,
        //     Some(_) => panic!("Unsupported supercompression format {:?}", format),
        // };


        // basis_universal::transcoder_init(); // TODO difference?
        // let mut transcoder = Transcoder::new();

        // if !basisu_transcoder.validate_header(&miniz_decoded_data) {
        //     anyhow::bail!("Image data failed basisu validation");
        // }

        // transcoder.user_data(image_bytes);
        // transcoder.prepare_transcoding(data.as_slice()).unwrap();
        // let result = transcoder.transcode_image_level(
        //     data.as_slice(),
        //     TranscoderTextureFormat::BC7_RGBA,
        //     TranscodeParameters {
        //         image_index: 0,
        //         level_index: 0,
        //         ..Default::default()
        //     },
        // ).unwrap();
        //
        // transcoder.end_transcoding();

        let asset = ImageAsset {
            name: image_name.to_string(),
            dimensions: TextureDimensions {
                pixel_width,
                pixel_height,
            },
            encoding,
            has_alpha,
            // data: data.to_vec(),
            data: result,
        };
        log::error!("Loaded image: {:?}", asset);
        asset
    }
}
