use cgmath::Point3;

pub fn distance_3d(point1: &Point3<f32>, point2: &Point3<f32>) -> f32 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2) + (point2.z - point1.z).powi(2))
        .sqrt()
}
