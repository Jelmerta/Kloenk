use crate::render::model::{ColoredVertex, Mesh, Model, TexVertex, VertexType};
use crate::render::texture;
use cfg_if::cfg_if;
use cgmath::Vector3;
use gltf::mesh::util::ReadIndices;
use gltf::Gltf;
use std::env;
use std::path::PathBuf;
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

    Ok(Model { meshes })
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
    match env::current_dir() {
        Ok(path) => log::warn!("{}", path.display()),
        Err(e) => log::warn!("{}", e),
    }
    let data = load_binary(model_path)
        .await
        .unwrap_or_else(|_| panic!("Path {} could not be found", model_path));
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
                let bin = load_binary(uri).await.unwrap();
                buffer_data.push(bin);
            }
        }
    }

    let mut meshes = Vec::new();
    for mesh in gltf.meshes() {
        mesh.primitives().for_each(|primitive| {
            let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

            let vertices = if let Some(vertex_attribute) = reader.read_positions() {
                let mut vertices = Vec::new();
                vertex_attribute.for_each(|vertex| {
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
                match read_indices {
                    ReadIndices::U8(iter) => {
                        iter.for_each(|index| indices.push(index as u16));
                    }
                    ReadIndices::U16(iter) => {
                        iter.for_each(|index| indices.push(index));
                    }
                    ReadIndices::U32(iter) => {
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
    Model { meshes }
}

#[allow(clippy::cast_possible_truncation)]
pub async fn load_model(
    device: &Device,
    model_to_load: &str,
    mesh_material_id: &str,
) -> anyhow::Result<Model> {
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

    Ok(Model { meshes })
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
            let path = env::var("OUT_DIR").map(|out_dir| PathBuf::from(out_dir) // target/debug or target/release
                    .ancestors()
                    .nth(3).unwrap().to_path_buf()).unwrap_or_else(|_| PathBuf::from(".")).join("assets")
                        .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
