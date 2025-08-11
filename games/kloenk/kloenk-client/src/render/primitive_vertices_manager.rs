use crate::render::model::ColorTextureVertex;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::Device;

pub struct PrimitiveVertices {
    pub name: String,
    pub vertices: Vec<ColorTextureVertex>,
    pub indices: Vec<u16>,
}

#[derive(Debug)]
pub struct PrimitiveVerticesGpu { // Probably need a better name, but mesh/model are already used. This means just the vertices of the mesh, without any texture
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

pub struct PrimitiveVerticesManager {
    primitive_vertices_map: HashMap<String, PrimitiveVerticesGpu>,
}

impl PrimitiveVerticesManager {
    pub fn new() -> PrimitiveVerticesManager {
        PrimitiveVerticesManager {
            primitive_vertices_map: HashMap::new(),
        }
    }

    pub fn get_primitive_vertices(&self, id: String) -> &PrimitiveVerticesGpu {
        self.primitive_vertices_map.get(&id).unwrap()
    }

    // pub fn load_primitive_vertices_to_memory(&mut self, device: &Device, id: String, vertices: &[TexVertex], indices: &[u16]) {
    pub fn load_primitive_vertices_to_memory(&mut self, device: &Device, primitive_vertices: PrimitiveVertices) {
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
        self.primitive_vertices_map.insert(primitive_vertices.name, primitive_vertices_gpu);
    }

    // TODO overload for colored vertices?
    //     device: &Device,
    //     model: &&[ColoredVertex],
    //     indices: &&[u16],
    //     color: Vector3<f32>,
}