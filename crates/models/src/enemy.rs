use super::*;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Enemy {
    pub id: Entity,
    pub aggro_radius: f32,
    pub target_position: Option<Vec3>,
    pub target_entity: Option<Entity>,

    pub attributes: Attributes,
    pub comp_attribs: ComputedAttributes,

    // pub animation_state: AnimationState,
    // pub animations: HashMap<String, AnimationNodeIndex>,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            id: Entity::PLACEHOLDER,
            // animation_state: AnimationState::StandIdle,
            // animations: HashMap::new(),
            aggro_radius: 14.0,
            target_position: None,
            target_entity: None,

            attributes: Attributes::default(),
            comp_attribs: ComputedAttributes::default(),
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
