pub struct GameState {
    pub entities: Vec<Entity>,
}

pub struct Entity {
    pub position: Position,
}

impl Entity {
    pub fn get_position(&self) -> &Position {
        &self.position
    }
}

pub struct Position {
    pub x: f32,
    pub y: f32,
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
            }
        };
        let enemy = Entity {
            position: Position {
                x: 1.0,
                y: 1.0,
            }
        };

        entities.push(player);
        entities.push(enemy);

        Self {
            entities: entities,
        }
    }

    pub fn get_entities(&self) -> &Vec<Entity> { // &?
        &self.entities
    }
}
