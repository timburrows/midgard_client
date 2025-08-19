use super::*;

pub fn get_capsule_radius(collider: &Collider) -> Option<f32> {
    collider.shape().0.as_capsule().map(|c| c.radius)
}
