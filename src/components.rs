use cgmath::Point3;

pub type Entity = String;

pub struct Graphics3D {
    pub model_id: String,
}

pub struct Graphics2D {
    pub model_id: String,
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

// Does not contain any data (yet?)
// pub struct Surface {}

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

// //Health?
// pub struct Resource {
//     amount_of_items: u8,
//     gathering_chance: f32,
//     received_item: ItemDefinition,
// }
// // Droptable?
//
// // Should only be defined once for every type of item, no need to copy
// // Basically a blueprint to construct items with its components
// pub struct ItemDefinition {
//     name: String,
//     model_2d: String,
//     model_3d: String,
//     hitbox: f32,
// }
//
// pub struct Action {
//     pub action_type: ActionType,
//     target: Entity,
// }
//
// pub enum ActionType {
//     Woodcutting,
// }
//
//
//
// // the possible position states are separate component types. Some possible position states:
//     // MapLocation - where it is on the map.
//     // Carried - indicating the entity carrying it.
//     // Equipped - indicating who is using it.
//     // InContainer - inside a treasure chest.
