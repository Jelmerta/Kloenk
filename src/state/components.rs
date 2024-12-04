use cgmath::Point3;

pub type Entity = String;

pub struct Graphics3D {
    pub mesh_id: String,
}

pub struct Graphics2D {
    pub material_id: String,
}

pub struct Storable {
    pub shape: ItemShape,
}

pub struct Storage {
    pub number_of_rows: u8,
    pub number_of_columns: u8,
}

pub struct InStorage {
    pub storage_entity: Entity,
    pub position_x: u8,
    pub position_y: u8,
}

#[derive(Clone)]
pub struct ItemShape {
    pub width: u8,
    pub height: u8,
}

pub struct Hitbox {
    pub box_corner_min: Point3<f32>,
    pub box_corner_max: Point3<f32>,
}

#[derive(Clone, Copy)]
pub struct CameraTarget {
    pub distance: f32,
    pub rotation_x_degrees: f32, // Spherical coordinates
    pub rotation_y_degrees: f32,
}

pub struct Size {
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}
