use crate::render::model::ColorTextureVertex;
use std::collections::HashMap;
use std::convert::Into;
use wgpu::util::DeviceExt;
use wgpu::Device;

const CUBE_TEX: &[ColorTextureVertex] = &[
    // Top ccw as seen from top
    ColorTextureVertex {
        position: [0.5, 0.5, 0.5],
        tex_coords: [0.0, 1.0],
    },
    ColorTextureVertex {
        position: [0.5, 0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    ColorTextureVertex {
        position: [-0.5, 0.5, -0.5],
        tex_coords: [1.0, 0.0],
    },
    ColorTextureVertex {
        position: [-0.5, 0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    // Bottom ccw as seen from top
    ColorTextureVertex {
        position: [0.5, -0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    ColorTextureVertex {
        position: [0.5, -0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    ColorTextureVertex {
        position: [-0.5, -0.5, -0.5],
        tex_coords: [1.0, 0.0],
    },
    ColorTextureVertex {
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

const SQUARE_TEX: &[ColorTextureVertex] = &[
    ColorTextureVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    ColorTextureVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    ColorTextureVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    ColorTextureVertex {
        position: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

const SQUARE_INDICES: &[u16] = &[2, 1, 0, 3, 2, 0];

pub struct PrimitiveVertices {
    pub name: String,
    pub vertices: Vec<ColorTextureVertex>,
    pub indices: Vec<u16>,
}

#[derive(Debug)]
pub struct PrimitiveVerticesGpu {
    // Probably need a better name, but mesh/model are already used. This means just the vertices of the mesh, without any texture
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

pub struct PrimitiveVerticesManager {
    primitive_vertices_map: HashMap<String, PrimitiveVerticesGpu>,
}

impl PrimitiveVerticesManager {
    pub fn new(device: &Device) -> PrimitiveVerticesManager {
        let mut primitive_vertices_manager = PrimitiveVerticesManager {
            primitive_vertices_map: HashMap::new(),
        };

        let square = PrimitiveVertices {
            name: "SQUARE".to_owned(),
            vertices: SQUARE_TEX.into(),
            indices: SQUARE_INDICES.to_vec(),
        };
        primitive_vertices_manager.load_primitive_vertices_to_memory(device, square);

        let cube = PrimitiveVertices {
            name: "CUBE".to_owned(),
            vertices: CUBE_TEX.into(),
            indices: CUBE_INDICES.to_vec(),
        };
        primitive_vertices_manager.load_primitive_vertices_to_memory(device, cube);

        primitive_vertices_manager
    }

    pub fn get_primitive_vertices(&self, id: &str) -> &PrimitiveVerticesGpu {
        self.primitive_vertices_map.get(id).unwrap()
    }

    // pub fn load_primitive_vertices_to_memory(&mut self, device: &Device, id: String, vertices: &[TexVertex], indices: &[u16]) {
    pub fn load_primitive_vertices_to_memory(
        &mut self,
        device: &Device,
        primitive_vertices: PrimitiveVertices,
    ) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&primitive_vertices.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&primitive_vertices.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = primitive_vertices.indices.len() as u32;
        let primitive_vertices_gpu = PrimitiveVerticesGpu {
            vertex_buffer,
            index_buffer,
            num_indices,
        };
        self.primitive_vertices_map
            .insert(primitive_vertices.name, primitive_vertices_gpu);
    }

    // TODO overload for colored vertices?
    //     device: &Device,
    //     model: &&[ColoredVertex],
    //     indices: &&[u16],
    //     color: Vector3<f32>,
}
