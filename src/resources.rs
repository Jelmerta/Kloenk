use cfg_if::cfg_if;

use wgpu::util::DeviceExt;
//
use crate::{model, texture};

const CUBE_TEX: &[model::TexVertex] = &[
    // Top ccw as seen from top
    model::TexVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
    }, // Red
    model::TexVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
    }, // Green
    model::TexVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // Yellow
    model::TexVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // Purple
    // Bottom ccw as seen from top
    model::TexVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
    }, // Blue
    model::TexVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    }, // Cyan
    model::TexVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
    }, // Black
    model::TexVertex {
        position: [-0.5, -0.5, 0.5],
        tex_coords: [1.0, 1.0],
    }, // White
];

const CUBE_INDICES: &[u16] = &[
    // Top
    0, 1, 2, 0, 2, 3, // Bottom
    4, 7, 6, 4, 6, 5, // Left
    0, 3, 7, 0, 7, 4, // Right
    1, 6, 2, 1, 5, 6, // Front
    0, 4, 5, 0, 5, 1, // Back
    2, 6, 7, 2, 7, 3,
];

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let diffuse_texture = load_texture(file_name, device, queue).await?;
    let bind_group = build_bind_group(device, layout, &diffuse_texture);

    let mut materials = Vec::new();
    materials.push(model::Material {
        name: file_name.to_string(),
        diffuse_texture,
        bind_group,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(CUBE_TEX),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(CUBE_INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = CUBE_INDICES.len() as u32;

    let mut meshes = Vec::new();
    meshes.push(model::Mesh {
        name: file_name.to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    });

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
        layout: &texture_bind_group_layout,
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

    return diffuse_bind_group;
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

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
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("resources")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
