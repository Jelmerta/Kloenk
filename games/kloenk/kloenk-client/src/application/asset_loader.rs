use crate::application::asset_loader::AssetType::Image;
use hydrox::load_binary;
use std::io::{Cursor, Read};

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

// TODO should probably contain transcoded image?
#[derive(Debug)]
pub enum ImageEncoding {
    BasisLz,
    Uastc,
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

    // Helpful: https://github.com/woelper/oculante/blob/680faabd105435b7c2668bcd3be715e28aa9605e/src/ktx2_loader/ktx2.rs
    fn load_image(image_name: &str, image_bytes: &[u8]) -> ImageAsset {
        log::error!("Loading image: {}", image_name);
        let reader = Reader::new(image_bytes).unwrap();

        // Handle supercompression TODO handle etc1s?
        let decompressed_data = Self::handle_supercompression(&reader);
        log::error!("Decompressed data: {:?}", decompressed_data);
        // Verify Basis header
        // if !decompressed_data.starts_with(&[0x73, 0x42, 0x61, 0x73]) {
        //     panic!("Invalid Basis file signature in first level");
        // }

        // Load GPU format. Note: Header format is empty when supercompression is used. We instead look at information in DFD block to decide how to handle format
        log::error!("Loading format: {:?}", reader.header());
        let pixel_width = reader.header().pixel_width;
        let pixel_height = reader.header().pixel_height;
        let dfd_block = reader.dfd_blocks().next().unwrap(); // TODO are there many? could some have other DFD model? basically only need to check one value which should be the same for the whole image?

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

        // // ETC1 a subset of ETC2 only supporting Rgb8
        // Some(ColorModel::ETC1) => {
        //         TextureFormat::Etc2Rgb8UnormSrgb
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


        // TODO i probably just need lowleveluastc...
        // let mut transcoder = Transcoder::new();
        // // transcoder.prepare_transcoding(&decompressed_data).map_err(|err| log::error!("{:?}", err)).ok();
        // let to_transcode: &[u8] = &decompressed_data;
        // let file_info = transcoder.file_info(to_transcode).unwrap();
        // log::error!("file: {:?}", file_info);
        // let info = transcoder.image_info(to_transcode, 0).unwrap();
        // log::error!("Image info: {:?}", info);
        // let level_info = transcoder.image_level_info(to_transcode, 0, 0).unwrap();
        // log::error!("Level info: {:?}", level_info);
        // transcoder.prepare_transcoding(to_transcode).map_err(|err| log::error!("wat {:?}", err)).ok();
        // let result1 = transcoder
        //     .transcode_image_level(
        //         to_transcode,
        //         TranscoderTextureFormat::BC7_RGBA,
        //         TranscodeParameters {
        //             image_index: 0,
        //             level_index: 0,
        //             decode_flags: Some(DecodeFlags::HIGH_QUALITY), // TODO alpha?
        //             output_row_pitch_in_blocks_or_pixels: None,
        //             output_rows_in_pixels: None,
        //         },
        //     )
        //     .unwrap();
        // log::error!("result1: {:?}", result1.len());


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
            data: decompressed_data, // todo wrong format
        };
        log::error!("Loaded image: {:?}", asset);
        asset
    }

    fn handle_supercompression(reader: &Reader<&[u8]>) -> Vec<u8> {
        let supercompress_scheme = reader.header().supercompression_scheme;
        if reader.header().supercompression_scheme.is_some() {
            let scheme = supercompress_scheme.unwrap();

            match scheme {
                SupercompressionScheme::Zstandard => {
                    // let data_offset = reader.data()
                    let level = reader.levels().next().unwrap(); // Assume 1 level
                    log::error!("Supercompression level byte: {:?}", level.uncompressed_byte_length);
                    log::error!("Supercompression level len: {:?}", level.data.bytes().count());
                    log::error!("Supercompression level len: {:?}", level.data.bytes());
                    log::error!("offset {:?}",                     reader.header().index.dfd_byte_offset);
                    log::error!("offset {:?}",                     reader.header().index);

                    let mut cursor = Cursor::new(level.data);
                    let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut cursor).unwrap();
                    let mut decompressed = Vec::new();
                    decoder.read_to_end(&mut decompressed).map_err(|err| {
                        log::error!(
                            "Failed to decompress",
                        )
                    }).unwrap();

                    // Validate decompressed size matches header
                    if reader.header().supercompression_scheme != Some(SupercompressionScheme::BasisLZ) {
                        assert_eq!(
                            decompressed.len(),
                            level.uncompressed_byte_length as usize,
                            "Decompressed size mismatch"
                        );
                    }

                    decompressed

                    // decompress(
                    //     level.data,
                    //     level.uncompressed_byte_length.to_usize().unwrap(),
                    // )
                    //     .unwrap()
                }
                SupercompressionScheme::BasisLZ => reader.levels().next().unwrap().data.to_vec(),
                _ => panic!(
                    "Unsupported supercompression scheme: {:?}",
                    supercompress_scheme
                ),
            }
        } else {
            panic!(
                "Unsupported supercompression scheme: {:?}",
                supercompress_scheme
            );
        }

        // else {
        // panic ! ("Unsupported supercompression scheme: {:?}", supercompress_scheme);
        // }
        // }
    }

    //     for best practice Transcoding requires device/queue to be loaded i think in order to dynamically figure transcoding format https://github.com/woelper/oculante/blob/master/src/ktx2_loader/image.rs#L387
    //         pub fn from_features(features: wgpu::Features) -> Self {
    //         let mut supported_compressed_formats = Self::default();
    //         if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) {
    //             supported_compressed_formats |= Self::ASTC_LDR;
    //         }
    //         if features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
    //             supported_compressed_formats |= Self::BC;
    //         }
    //         if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) {
    //             supported_compressed_formats |= Self::ETC2;
    //         }
    //         supported_compressed_formats
    //     }

    //     Similarly?
    //     // NOTE: Rgba16Float should be transcoded to BC6H/ASTC_HDR. Neither are supported by
    //         // basis-universal, nor is ASTC_HDR supported by wgpu
    //         DataFormat::Rgb | DataFormat::Rgba => {
    //             // NOTE: UASTC can be losslessly transcoded to ASTC4x4 and ASTC uses the same
    //             // space as BC7 (128-bits per 4x4 texel block) so prefer ASTC over BC for
    //             // transcoding speed and quality.
    //             if supported_compressed_formats.contains(CompressedImageFormats::ASTC_LDR) {
    //                 (
    //                     TranscoderBlockFormat::ASTC_4x4,
    //                     TextureFormat::Astc {
    //                         block: AstcBlock::B4x4,
    //                         channel: if is_srgb {
    //                             AstcChannel::UnormSrgb
    //                         } else {
    //                             AstcChannel::Unorm
    //                         },
    //                     },
    //                 )
    //             } else if supported_compressed_formats.contains(CompressedImageFormats::BC) {
    //                 (
    //                     TranscoderBlockFormat::BC7,
    //                     if is_srgb {
    //                         TextureFormat::Bc7RgbaUnormSrgb
    //                     } else {
    //                         TextureFormat::Bc7RgbaUnorm
    //                     },
    //                 )
    //             } else if supported_compressed_formats.contains(CompressedImageFormats::ETC2) {
    //                 (
    //                     TranscoderBlockFormat::ETC2_RGBA,
    //                     if is_srgb {
    //                         TextureFormat::Etc2Rgba8UnormSrgb
    //                     } else {
    //                         TextureFormat::Etc2Rgba8Unorm
    //                     },
    //                 )
    //             } else {
    //                 (
    //                     TranscoderBlockFormat::RGBA32,
    //                     if is_srgb {
    //                         TextureFormat::Rgba8UnormSrgb
    //                     } else {
    //                         TextureFormat::Rgba8Unorm
    //                     },
    //                 )
    //             }
    //         }
    //     }
}
