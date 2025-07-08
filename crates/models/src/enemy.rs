use super::*;
use std::collections::HashMap;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Enemy {
    pub id: Entity,
    pub speed: f32,
    pub aggro_radius: f32,
    pub target_position: Option<Vec3>,
    pub target_entity: Option<Entity>,
    // pub animation_state: AnimationState,
    // pub animations: HashMap<String, AnimationNodeIndex>,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            id: Entity::PLACEHOLDER,
            speed: 1.0,
            // animation_state: AnimationState::StandIdle,
            // animations: HashMap::new(),
            aggro_radius: 14.0,
            target_position: None,
            target_entity: None,
        }
    }
}

// #[derive(Component, Reflect, Default, Clone)]
// #[reflect(Component)]
// pub enum EnemyAnimationState {
//     #[default]
//     StandIdle,
//     Run(f32),
//     Sprint(f32),
//     Climb(f32),
//     JumpStart,
//     JumpLoop,
//     JumpLand,
//     Fall,
//     Crawl(f32),
//     Crouch,
//     Dash,
//     WallSlide,
//     WallJump,
//     KnockBack,
// }
