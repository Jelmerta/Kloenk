use crate::components::Hitbox;

pub struct CollisionManager {}

impl CollisionManager {
    pub fn check_collision(bounding_box_one: &Hitbox, bounding_box_two: &Hitbox) -> bool {
        if bounding_box_one.box_corner_max.x <= bounding_box_two.box_corner_min.x
            || bounding_box_one.box_corner_min.x >= bounding_box_two.box_corner_max.x
        {
            return false;
        }

        if bounding_box_one.box_corner_max.y <= bounding_box_two.box_corner_min.y
            || bounding_box_one.box_corner_min.y >= bounding_box_two.box_corner_max.y
        {
            return false;
        }

        if bounding_box_one.box_corner_max.z <= bounding_box_two.box_corner_min.z
            || bounding_box_one.box_corner_min.z >= bounding_box_two.box_corner_max.z
        {
            return false;
        }

        true
    }

    pub fn check_in_dimension(
        position1: f32,
        boundary1: f32,
        position2: f32,
        boundary2: f32,
    ) -> bool {
        position1 + boundary1 >= position2 - boundary2
            && position2 + boundary2 >= position1 - boundary1
    }
}
