use super::*;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_attack_event.run_if(in_state(Screen::Gameplay)),
    );

    app.add_event::<AttackEvent>();
}

#[derive(Component, Debug)]
pub struct AttackRateTimer(Timer);

impl AttackRateTimer {
    // This timer starts in the finished state, so that finished() may be checked
    // before an attack lands, and begin ticking afterward
    pub fn new(secs: f32) -> Self {
        let mut timer = Timer::from_seconds(secs, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(secs + 1.0));
        Self(timer)
    }
}

#[derive(Event, Debug)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub target: Entity,
}

fn handle_attack_event(
    mut attack_event: EventReader<AttackEvent>,
    mut combatant_query: Query<(
        &Transform,
        &Collider,
        &mut ComputedAttributes,
        Option<&mut AttackRateTimer>,
    )>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    let Some(attack_evt) = attack_event.read().next() else {
        return;
    };

    let Ok([attacker_bundle, target_bundle]) =
        combatant_query.get_many_mut([attack_evt.attacker, attack_evt.target])
    else {
        return;
    };

    let (attacker_transform, attacker_collider, attacker_attribs, attacker_timer_opt) = attacker_bundle;
    let (target_transform, target_collider, mut target_attribs, _) = target_bundle;
    
    let dist = attacker_transform
        .translation
        .xz()
        .distance(target_transform.translation.xz());

    let attacker_radius = get_capsule_radius(attacker_collider);
    let target_radius = get_capsule_radius(target_collider);

    const COLLISION_BUFFER: f32 = 0.5;
    let collider_dist = attacker_radius + target_radius + COLLISION_BUFFER;
    let attack_range = attacker_attribs.attack_range;
    let is_in_range = dist <= collider_dist + attack_range;

    if is_in_range {
        if let Some(mut timer) = attacker_timer_opt {
            if !timer.0.finished() {
                timer.0.tick(time.delta());
            }

            if timer.0.finished() {
                let attack_damage = attacker_attribs.attack;
                target_attribs.health.hp -= attack_damage;

                info!(
                    "Deals {} damage, remaining health {}",
                    attack_damage, target_attribs.health.hp
                );

                if target_attribs.health.hp <= 0.0 {
                    info!("Target has died");
                    cmd.entity(attack_evt.target).despawn();
                } else {
                    timer.0.reset();
                }
            }
        }
    }
}

// fixme: this doesn't really belong here if we are going to treat this as shared code
pub fn get_capsule_radius(collider: &Collider) -> f32 {
    let shape = collider.shape();
    let capsule_result = shape.0.as_capsule();

    match capsule_result {
        Some(capsule) => capsule.radius,
        None => {
            error!("Entity collider is not a capsule shape");
            0.0
        }
    }
}
