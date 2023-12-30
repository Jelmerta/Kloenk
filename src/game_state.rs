pub struct GameState {
    pub player: Entity,
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
            entities: entities,
        }
    }

    pub fn get_entities(&self) -> &Vec<Entity> { // &?
        &self.entities
    }
}
