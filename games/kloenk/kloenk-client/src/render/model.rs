use crate::render::primitive_vertices_manager::PrimitiveVerticesGpu;
use cgmath::Vector3;
use std::ops::Range;

pub trait Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorTextureVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}


impl Vertex for ColorTextureVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColorTextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct ModelToLoad {
    pub name: String,
    pub primitives_to_load: Vec<PrimitiveToLoad>,
}

pub struct PrimitiveToLoad {
    pub vertices_to_load: String, // todo some data can be in-memory already? maybe should just start modelloader with square/cube. interpreted as gltf file?
    pub material_to_load: MaterialToLoad,
}

// or maybe 1x1 vs file
pub enum MaterialToLoad {
    Color { name: String, rgb: Vector3<f32> },
    Texture { file_name: String },
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Primitive {
    pub primitive_vertices_id: String,
    pub material_id: String,
}

// TODO isn't this maybe too similar to ModelToLoad?...
pub struct Model {
    pub primitives: Vec<Primitive>,
}


// sotrh decides to implement a trait on renderpass
pub trait Draw<'a> {
    fn draw_primitive_instanced(&mut self, mesh: &'a PrimitiveVerticesGpu, instances: Range<u32>);
}

// TODO look at lifetimes...
impl<'a, 'b> Draw<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    // fn draw_primitive_instanced(&mut self, primitive: &'b Primitive, instances: Range<u32>) {
    fn draw_primitive_instanced(&mut self, primitive_vertices: &'b PrimitiveVerticesGpu, instances: Range<u32>) {
        self.set_vertex_buffer(0, primitive_vertices.vertex_buffer.slice(..));
        // TODO 16 or 32
        self.set_index_buffer(primitive_vertices.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..primitive_vertices.num_indices, 0, instances);
    }
}
