use super::*;
use std::collections::HashMap;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Player {
    pub id: Entity,
    pub speed: f32,
    pub animation_state: AnimationState,
    pub animations: HashMap<String, AnimationNodeIndex>,
    pub target_position: Option<Vec3>,

    pub attribs: Attributes,
    pub comp_attribs: ComputedAttributes,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: Entity::PLACEHOLDER,
            speed: 1.0,
            animation_state: AnimationState::StandIdle,
            animations: HashMap::new(),
            target_position: None,
            
            attribs: Attributes::default(),
            comp_attribs: ComputedAttributes::default(),
        }
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub enum AnimationState {
    #[default]
    StandIdle,
    Run(f32),
    Sprint(f32),
    Climb(f32),
    JumpStart,
    JumpLoop,
    JumpLand,
    Fall,
    Crawl(f32),
    Crouch,
    Dash,
    WallSlide,
    WallJump,
    KnockBack,
}
