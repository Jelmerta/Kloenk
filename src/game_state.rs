pub const TOTAL_DISTANCE: f32 = 200000.; // Verify naming, probbaly not total distance

pub struct GameState {
    pub player: Entity,
    pub camera: Entity,
    pub entities: Vec<Entity>,
}

pub struct Entity {
    pub position: Position,
    pub previous_position: Position,
    pub hitbox: f32,
}

impl Entity {
    pub fn get_position(&self) -> &Position {
        &self.position
    }
}

pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut entities = Vec::new();
        let player = Entity {
            position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            previous_position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            hitbox: 0.5,
        };

        let starting_camera_distance: f32 = f32::sqrt(TOTAL_DISTANCE / 3.0);
        let camera = Entity {
            previous_position: Position { // previous position probably not really used right now
                // for camera.
                x: starting_camera_distance,
                y: starting_camera_distance,
                z: starting_camera_distance,
            },
            position: Position {
                x: starting_camera_distance,
                y: starting_camera_distance,
                z: starting_camera_distance,
            },
            hitbox: 0.0, // Doesn't make sense for a camera... Not sure if camera really belongs
            // here. Might make hitbox a trait or something like that too
        };

        let enemy = Entity {
            position: Position {
                x: 1.1,
                y: 1.1,
                z: 0.0,
            },
            previous_position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            hitbox: 0.51,
        };

        entities.push(enemy);

        Self {
            player: player,
            camera: camera,
            entities: entities,
        }
    }

    pub fn get_entities(&self) -> &Vec<Entity> { // &?
        &self.entities
    }
}
