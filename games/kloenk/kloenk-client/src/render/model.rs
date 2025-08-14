use crate::render::primitive_vertices_manager::PrimitiveVerticesGpu;
use cgmath::Vector4;
use std::ops::Range;

pub trait Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorTextureVertex {
    pub position: [f32; 3],
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
                // tex coords
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorDefinition {
    pub id: String,
    pub value: Vector4<f32>,
}

#[derive(Debug, Clone)]
pub struct TextureDefinition {
    pub id: String,
    pub file_name: String,
}

pub struct ModelDefinition {
    pub id: String,
    pub primitives: Vec<PrimitiveDefinition>,
}

#[derive(Debug, Clone)]
pub struct PrimitiveDefinition {
    // todo primitives usually dont have a name, what do we decide? gozer_primitive_1? separate from vertices?
    pub vertices_id: String, // todo some data can be in-memory already? maybe should just start modelloader with square/cube. interpreted as gltf file?
    pub color_definition: ColorDefinition, // If just displaying texture for example, can be set to white
    pub texture_definition: Option<TextureDefinition>,
}

//todo keep in mind sharing textures/colors
// pub enum MaterialToLoad {
//     Color { name: String, rgb: Vector3<f32> },
//     Texture { file_name: String },
// }

// #[derive(Debug)]
// #[derive(Clone)]
// pub struct Primitive {
//     pub primitive_vertices_id: String,
//     pub color: String,
//     pub material_id: String,
// }

// TODO isn't this maybe too similar to ModelToLoad?...
// pub struct Model {
//     pub primitives: Vec<Primitive>,
// }

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
    fn draw_primitive_instanced(
        &mut self,
        primitive_vertices: &'b PrimitiveVerticesGpu,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, primitive_vertices.vertex_buffer.slice(..));
        // TODO 16 or 32
        self.set_index_buffer(
            primitive_vertices.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        self.draw_indexed(0..primitive_vertices.num_indices, 0, instances);
    }
}
