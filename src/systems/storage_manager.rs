use crate::state::components::{Entity, ItemShape, Storage};
use crate::state::game_state::GameState;

pub struct StorageManager {}

impl StorageManager {
    pub fn has_space(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
        near_pickup: &Entity,
    ) -> bool {
        Self::find_empty_spot(game_state, storage, in_storage_entities, near_pickup).is_some()
    }

    pub fn find_empty_spot(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
        near_pickup: &Entity,
    ) -> Option<(u8, u8)> {
        let dynamic_storage =
            Self::generate_dynamic_storage_space(game_state, storage, in_storage_entities);
        let item_shape = &game_state
            .storable_components
            .get(near_pickup)
            .unwrap()
            .shape;
        let mut padded_storage = vec![vec![true; 12]; 12];
        for x in 0..dynamic_storage.len() {
            for y in 0..dynamic_storage.len() {
                padded_storage[y][x] = dynamic_storage[y][x];
            }
        }

        for row in 0..storage.number_of_rows {
            for column in 0..storage.number_of_columns {
                if Self::check_empty_spot(&padded_storage, row, column, item_shape) {
                    return Some((column, row));
                }
            }
        }
        None
    }

    fn check_empty_spot(
        padded_storage: &[Vec<bool>],
        row: u8,
        column: u8,
        shape: &ItemShape,
    ) -> bool {
        for x in column..column + shape.width {
            for y in row..row + shape.height {
                if padded_storage[y as usize][x as usize] {
                    return false;
                }
            }
        }
        true
    }

    fn generate_dynamic_storage_space(
        game_state: &GameState,
        storage: &Storage,
        in_storage_entities: &Vec<&Entity>,
    ) -> Vec<Vec<bool>> {
        let mut storage_spots =
            vec![vec![false; storage.number_of_rows.into()]; storage.number_of_columns.into()];

        for in_storage_entity in in_storage_entities {
            let in_storage = game_state
                .in_storage_components
                .get(&(*in_storage_entity).to_string())
                .unwrap();
            let storable = game_state
                .storable_components
                .get(&(*in_storage_entity).to_string())
                .unwrap();
            for x in in_storage.position_x..in_storage.position_x + storable.shape.width {
                for y in in_storage.position_y..in_storage.position_y + storable.shape.height {
                    storage_spots[y as usize][x as usize] = true;
                }
            }
        }
        storage_spots
    }

    pub fn get_in_storage<'a>(game_state: &'a GameState, entity: &Entity) -> Vec<&'a Entity> {
        game_state
            .entities
            .iter()
            .filter(|e| {
                game_state
                    .in_storage_components
                    .contains_key(&(*e).to_string())
            })
            .filter(|e| {
                game_state
                    .in_storage_components
                    .get(&(*e).to_string())
                    .unwrap()
                    .storage_entity
                    == *entity
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_in_storage_entities<'a>(
        game_state: &'a GameState,
        entity: &Entity,
    ) -> Vec<&'a Entity> {
        game_state
            .entities
            .iter()
            .filter(|e| {
                game_state
                    .in_storage_components
                    .contains_key(&(*e).to_string())
            })
            .filter(|e| {
                game_state
                    .in_storage_components
                    .get(&(*e).to_string())
                    .unwrap()
                    .storage_entity
                    == *entity
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn find_in_storage<'a>(game_state: &'a GameState, entity: &Entity) -> Option<&'a Entity> {
        let storage_entities = StorageManager::get_in_storage_entities(game_state, entity);
        storage_entities.first().copied()
    }
}
