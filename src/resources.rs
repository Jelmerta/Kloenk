use crate::render::model::{ColoredVertex, Mesh, Model, TexVertex, VertexType};
use crate::render::{model, texture};
use cfg_if::cfg_if;
use cgmath::Vector3;
use gltf::accessor::Dimensions::Vec3;
use gltf::mesh::util::ReadIndices;
use gltf::Gltf;
use wgpu::util::DeviceExt;
use wgpu::Device;

const CUBE_TEX: &[TexVertex] = &[
    // Top ccw as seen from top
    TexVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
    },
    TexVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    TexVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    },
    TexVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    // Bottom ccw as seen from top
    TexVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    TexVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    TexVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
    },
    TexVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 1.0],
    },
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // Bottom
    4, 7, 6, 4, 6, 5, // Left
    0, 3, 7, 0, 7, 4, // Right
    1, 6, 2, 1, 5, 6, // Front
    0, 4, 5, 0, 5, 1, // Back
    2, 6, 7, 2, 7, 3,
];

const SQUARE_TEX: &[TexVertex] = &[
    TexVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    TexVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    TexVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    TexVertex {
        position: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

const SQUARE_INDICES: &[u16] = &[2, 1, 0, 3, 2, 0];

pub fn load_colored_square_model(device: &Device, color: Vector3<f32>) -> anyhow::Result<Model> {
    let model = load_colored_square(color);
    let indices = SQUARE_INDICES;

    let meshes = build_colored_meshes(device, &&model[..], &indices, color);

    Ok(model::Model { meshes })
}

fn load_colored_square(color: Vector3<f32>) -> Vec<ColoredVertex> {
    vec![
        ColoredVertex {
            position: [0.0, 0.0, 0.0],
            color: color.into(),
        },
        ColoredVertex {
            position: [1.0, 0.0, 0.0],
            color: color.into(),
        },
        ColoredVertex {
            position: [1.0, -1.0, 0.0],
            color: color.into(),
        },
        ColoredVertex {
            position: [0.0, -1.0, 0.0],
            color: color.into(),
        },
    ]
}

pub async fn load_gltf(device: &Device, model_path: &str) -> Model {
    let data = load_binary(model_path).await.unwrap();
    let gltf = Gltf::from_slice(data.as_slice())
        .unwrap_or_else(|_| panic!("Failed to load gltf model {}", model_path));

    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            // Think this is for if we want to load glb files?
            gltf::buffer::Source::Bin => {
                // if let Some(blob) = gltf.blob.as_deref() {
                //     buffer_data.push(blob.into());
                //     println!("Found a bin, saving");
                // };
            }
            // Think this is for if we want to load gltf+bin files?
            gltf::buffer::Source::Uri(uri) => {
                log::warn!("{}", uri);
                let bin = load_binary(uri).await.unwrap();
                buffer_data.push(bin);
            }
        }
    }

    let mut meshes = Vec::new();
    for mesh in gltf.meshes() {
        mesh.primitives().for_each(|primitive| {
            let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

            let vertices = if let Some(vertex_attibute) = reader.read_positions() {
                let mut vertices = Vec::new();
                vertex_attibute.for_each(|vertex| {
                    vertices.push(ColoredVertex {
                        position: vertex,
                        color: Vector3::new(0.7, 0.7, 0.7).into(),
                    })
                });
                vertices
            } else {
                Vec::new()
            };

            let indices = if let Some(read_indices) = reader.read_indices() {
                let mut indices = Vec::new();
                // indices.append(&mut read_indices.into_u32().collect::<Vec<u32>>());
                // indices.append(&mut read_indices.into_u32().collect::<Vec<u32>>());
                // indices
                match read_indices {
                    ReadIndices::U8(iter) => {
                        log::warn!("8");
                    }
                    ReadIndices::U16(iter) => {
                        log::warn!("16");
                        iter.for_each(|index| indices.push(index));
                    }
                    ReadIndices::U32(iter) => {
                        log::warn!("32");
                        iter.for_each(|index| indices.push(index as u16));
                    }
                }
                indices
            } else {
                Vec::new()
            };

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            // TODO Not used... Vertex buffer is already set containing the colors
            let vertex_type = if primitive.material().index().is_some() {
                VertexType::Color {
                    color: primitive.material().emissive_factor(),
                }
            } else {
                // No material found for the primitive, so just default to a color
                VertexType::Color {
                    color: Vector3::new(0.7, 0.7, 0.7).into(),
                }
            };
            meshes.push(Mesh {
                vertex_type,
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
            })
        });
    }
    log::warn!("{:?}", meshes);
    Model { meshes }
}

// let mut buffer_data = Vec::new();
// for buffer in gltf.buffers() {
//     match buffer.source() {
//         // Think this is what we want if we use .gltf files
//         gltf::buffer::Source::Uri(uri) => {
//             // let uri = percent_encoding::percent_decode_str(uri)
//             // .decode_utf8()
//             // .unwrap();
//             // let uri = uri.as_ref();
//             // let buffer_bytes = match DataUri::parse(uri) {
//             //     Ok(data_uri) if VALID_MIME_TYPES.contains(&data_uri.mime_type) => {
//             // data_uri.decode()?
//             // }
//             // Ok(_) => return Err(GltfError::BufferFormatUnsupported),
//             // Err(()) => {
//             // TODO: Remove this and add dep
//             // let buffer_path = load_context.path().parent().unwrap().join(uri);
//             // load_context.read_asset_bytes(buffer_path).await?
//             // }
//             // };
//             // buffer_data.push();
//         }
//         // Think this is for if we want to load glb files?
//         gltf::buffer::Source::Bin => {
//             if let Some(blob) = gltf.blob.as_deref() {
//                 buffer_data.push(blob.into());
//             } else {
//                 panic!(":)");
//             }
//         }
//     }
// }

//
// gltf.materials()
// //
// // let mut materials = Vec::new();
// for obj_material in object_materials? {
// gltf.1

// let diffuse_texture = load_tex;
// let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//     layout,
//     entries: &[
//         wgpu::BindGroupEntry {
//             binding: 0,
//             resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
//         },
//         wgpu::BindGroupEntry {
//             binding: 1,
//             resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
//         },
//     ],
//     label: None,
// });

//         materials.push((model::Material {
//             name: ,
//             diffuse_texture,
//             bind_group,
//         });
//     }
//
//     let garfield = model::Model {
//         meshes: meshes,
//         materials: materials,
//     };
//     // models.push(garfield);
//
//     return Model {
//         meshes: gltf.meshes().collect(),
//     };
// }

#[allow(clippy::cast_possible_truncation)]
pub async fn load_model(
    device: &Device,
    model_to_load: &str,
    mesh_material_id: &str,
) -> anyhow::Result<model::Model> {
    let model: &[TexVertex];
    let indices: &[u16];
    if model_to_load.eq("CUBE") {
        model = CUBE_TEX;
        indices = CUBE_INDICES;
    } else {
        model = SQUARE_TEX;
        indices = SQUARE_INDICES;
    }

    // Also add this to material
    // name: file_name.to_string(),
    // diffuse_texture,

    let meshes = build_textured_meshes(device, &model, &indices, mesh_material_id);

    Ok(model::Model { meshes })
}

fn build_textured_meshes(
    device: &Device,
    model: &&[TexVertex],
    indices: &&[u16],
    mesh_material_name: &str,
) -> Vec<Mesh> {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(model),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = indices.len() as u32;

    let meshes = vec![Mesh {
        vertex_type: VertexType::Material {
            material_id: mesh_material_name.to_string(), // Currently we set all meshes of a model to the same material
        },
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    }];
    meshes
}

fn build_colored_meshes(
    device: &Device,
    model: &&[ColoredVertex],
    indices: &&[u16],
    color: Vector3<f32>,
) -> Vec<Mesh> {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(model),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = indices.len() as u32;

    let meshes = vec![Mesh {
        vertex_type: VertexType::Color {
            color: color.into(), // Currently we set all meshes of a model to the same color
        },
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    }];
    meshes
}

pub async fn load_texture(
    file_name: &str,
    device: &Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    let base = reqwest::Url::parse(&format!("{origin}/",)).unwrap();
    base.join("resources/").unwrap().join(file_name).unwrap()
}

#[allow(clippy::unused_async)] // Only used in wasm
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new(std::env::var("OUT_DIR").unwrap_or_else(|_| String::from(".")).as_str())
                .join("resources")
                .join(file_name);

            // #[cfg(debug_assertions)] {
                log::info!("{}", path.display());
            // }

            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
