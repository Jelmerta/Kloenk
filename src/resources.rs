//
use crate::render::model::{ColoredVertex, Mesh, TexVertex, VertexType};
use crate::render::{model, texture};
use cfg_if::cfg_if;
use cgmath::Vector3;
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

const SQUARE_BLACK: &[ColoredVertex] = &[
    ColoredVertex {
        position: [0.0, 0.0, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    ColoredVertex {
        position: [1.0, 0.0, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    ColoredVertex {
        position: [1.0, -1.0, 0.0],
        color: [0.0, 0.0, 0.0],
    },
    ColoredVertex {
        position: [0.0, -1.0, 0.0],
        color: [0.0, 0.0, 0.0],
    },
];

const SQUARE_INDICES: &[u16] = &[2, 1, 0, 3, 2, 0];

pub fn load_black_square_model(device: &Device) -> anyhow::Result<model::Model> {
    let model: &[ColoredVertex];
    let indices: &[u16];
    model = SQUARE_BLACK;
    indices = SQUARE_INDICES;

    let meshes = build_colored_meshes(device, &model, &indices, Vector3::new(0.0, 0.0, 0.0));

    Ok(model::Model { meshes })
}

pub fn load_colored_square(color: Vector3<f32>) -> Vec<ColoredVertex> {
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
