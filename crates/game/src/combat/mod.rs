use super::*;
use std::time::Duration;
use event::types::AttackEvent;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_attack_timers.run_if(in_state(Screen::Gameplay)),
            handle_attack_event.run_if(in_state(Screen::Gameplay)),
        ),
    );
}

#[derive(Component, Debug)]
pub struct AttackRateTimer(Timer);

impl AttackRateTimer {
    // This timer starts in the finished state, so that finished() may be checked
    // before an attack lands, and begin ticking afterward
    pub fn new(secs: f32) -> Self {
        let mut timer = Timer::from_seconds(secs, TimerMode::Once);
        timer.set_elapsed(Duration::from_secs_f32(secs));
        Self(timer)
    }
}

fn tick_attack_timers(mut q: Query<&mut AttackRateTimer>, time: Res<Time>) {
    for mut t in &mut q {
        t.0.tick(time.delta());
    }
}

fn handle_attack_event(
    mut ev: EventReader<AttackEvent>,
    mut combatant_q: Query<(
        &Transform,
        &Collider,
        &mut ComputedAttributes,
        Option<&Player>,
        &mut AttackRateTimer,
    )>,
    mut cmd: Commands,
) {
    for AttackEvent { attacker, target } in ev.read().copied() {
        let Ok([atkr_bundle, tgt_bundle]) = combatant_q.get_many_mut([attacker, target]) else {
            return;
        };

        let (atkr_tfm, atkr_collider, atkr_cattribs, atkr_is_player, mut atkr_atkrate_timer) =
            atkr_bundle;

        let (tgt_tfm, tgt_collider, mut tgt_cattribs, _tgt_enemy_opt, _tgt_atkrate_timer_opt) =
            tgt_bundle;

        if !is_in_attack_range(
            atkr_tfm,
            atkr_collider,
            atkr_cattribs.attack_range,
            tgt_tfm,
            tgt_collider,
        ) {
            continue;
        }

        // Cooldown gate
        if !atkr_atkrate_timer.0.finished() {
            continue;
        }

        // Apply damage
        let dmg = atkr_cattribs.attack;
        let died = apply_damage(&mut tgt_cattribs, dmg);

        let (atkr_type, tgt_type) = if let Some(_) = atkr_is_player {
            ("Player", "Enemy")
        } else {
            ("Enemy", "Player")
        };

        info!(
            "{atkr_type}({attacker}) deals {dmg} to {tgt_type}({target}); remaining hp {}",
            tgt_cattribs.health.hp
        );

        if died {
            info!("Target({target:?}) died");
            cmd.entity(target).despawn();
        } else {
            // Re-arm cooldown after a successful hit
            atkr_atkrate_timer.0.reset();
        }
    }
}

fn is_in_attack_range(
    atkr_tfm: &Transform,
    atkr_collider: &Collider,
    atkr_range: f32,
    tgt_tfm: &Transform,
    tgt_collider: &Collider,
) -> bool {
    const COLLISION_BUFFER: f32 = 0.5;

    let Some(atkr_radius) = utils::get_capsule_radius(atkr_collider) else {
        return false;
    };
    let Some(tgt_radius) = utils::get_capsule_radius(tgt_collider) else {
        return false;
    };

    let planar_dist = atkr_tfm.translation.xz().distance(tgt_tfm.translation.xz());
    let collider_dist = atkr_radius + tgt_radius + COLLISION_BUFFER;
    planar_dist <= collider_dist + atkr_range
}

// Returns true if target health drops to 0 or less
fn apply_damage(target: &mut ComputedAttributes, amount: f32) -> bool {
    target.health.hp -= amount;
    target.health.hp <= 0.0
}
