use super::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_event::<EnemyClickEvent>();
    app.add_event::<AttackEvent>();
    app.add_event::<ProximityEvent>();
    app.add_event::<PositionChangeEvent>();
    app.add_event::<GroundClickEvent>();
}

#[derive(Event)]
pub struct EnemyClickEvent {
    pub target: Entity,
    pub player: Entity,
}

#[derive(Event, Debug, Copy, Clone)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub target: Entity,
}

#[derive(Event)]
pub struct ProximityEvent {
    pub player_entity: Entity,
    pub enemy_entity: Entity,
}

#[derive(Event)]
pub struct PositionChangeEvent {
    pub entity: Entity,
    pub target: Entity,
}

impl PositionChangeEvent {
    pub fn new(entity: Entity, target: Entity) -> Self {
        Self { entity, target }
    }
}

#[derive(Event)]
pub struct GroundClickEvent {
    pub position: Vec3,
}