// List of components? component is attribute/eigenschap, not a concept not is-a, but has-a
// interactable
// position
// renderable? graphicscomponent?
// pickupable+droppable vs item?... shouldn't define behaviour but pickupable is more fine-grained
// and an attribute of the entity?

// what even is entity? A concept in the game, not attribute of something. is-a not has-a
// player
// npc
// monster
// spells?
// hmm quests?
// probably not inventory?


pub type Entity = String;

pub struct Graphics3D {
    pub model_id: String,
    pub material_id: String,
}

pub struct Graphics2D {
    pub model_id: String,
    pub material_id: String,
}

pub struct Storable {
//     pub shape: ItemShape,
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

// #[derive(Clone)]
// struct ItemShape { // vs StorageShape?
//     // shape: Vec<Vec<bool>>, // probably a better solution long term
//     width: u8,
//     height: u8,
// }

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Hitbox { // or physics or whatever
    pub hitbox: f32,
}

pub struct Walkable {
    
}

// Probably just a singleton... probably does not make much sense for this to be in the ECS?
pub struct CameraTarget {
    pub distance: f32,
    pub rotation_x_degrees: f32, // as seen on a sphere, to figure out the position of the camera. it is not
    // the direction the camera is pointed at
    pub rotation_y_degrees: f32,
}
