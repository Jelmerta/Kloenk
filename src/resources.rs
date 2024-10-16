//
use crate::{model, texture};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

const CUBE_TEX: &[model::TexVertex] = &[
    // Top ccw as seen from top
    model::TexVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
    },
    model::TexVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    model::TexVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    },
    model::TexVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    // Bottom ccw as seen from top
    model::TexVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    model::TexVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    model::TexVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
    },
    model::TexVertex {
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

const SQUARE_TEX: &[model::TexVertex] = &[
    model::TexVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    model::TexVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    model::TexVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    model::TexVertex {
        position: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

const SQUARE_INDICES: &[u16] = &[2, 1, 0, 3, 2, 0];

#[allow(clippy::cast_possible_truncation)]
pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    model_to_load: &str,
) -> anyhow::Result<model::Model> {
    let model: &[model::TexVertex];
    let indices: &[u16];
    if model_to_load.eq("CUBE") {
        model = CUBE_TEX;
        indices = CUBE_INDICES;
    } else {
        model = SQUARE_TEX;
        indices = SQUARE_INDICES;
    }

    let diffuse_texture = load_texture(file_name, device, queue).await?;
    let bind_group = build_bind_group(device, layout, &diffuse_texture);

    let materials = vec![model::Material { bind_group }];
    // Also add this to material
    // name: file_name.to_string(),
    // diffuse_texture,

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

    let meshes = vec![model::Mesh {
        // name: file_name.to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    }];

    Ok(model::Model { meshes, materials })
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

fn build_bind_group(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    diffuse_texture: &texture::Texture,
) -> wgpu::BindGroup {
    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    diffuse_bind_group
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
