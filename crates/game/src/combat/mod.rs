use super::*;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_attack_event.run_if(in_state(Screen::Gameplay)),
    );

    // app.add_systems(OnEnter(Screen::Gameplay), spawn_enemy);
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

fn handle_attack_event(
    mut enemy_query: Query<(&Enemy, &Transform, &mut AttackRateTimer, &Collider)>,
    mut player_query: Query<(&mut Player, &Transform, &Collider)>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    for (enemy, enemy_transform, mut attack_timer, enemy_collider) in enemy_query.iter_mut() {
        for (mut player, player_transform, player_collider) in player_query.iter_mut() {
            let dist = enemy_transform
                .translation
                .xz()
                .distance(player_transform.translation.xz());

            let player_radius = player_collider
                .shape()
                .0
                .as_capsule()
                .map_or_else(|| default(), |c| c.radius);

            let enemy_radius = enemy_collider
                .shape()
                .0
                .as_capsule()
                .map_or_else(|| default(), |c| c.radius);

            const COLLISION_BUFFER: f32 = 0.5;
            let collision_dist = enemy_radius + player_radius + COLLISION_BUFFER;
            let is_in_range = dist <= collision_dist + enemy.comp_attribs.attack_range;

            if is_in_range {
                if !attack_timer.0.finished() {
                    attack_timer.0.tick(time.delta());
                }

                if attack_timer.0.finished() {
                    player.comp_attribs.health.hp -= enemy.comp_attribs.attack;

                    info!(
                        "Enemy deals {} damage to Player, remaining health {}",
                        enemy.comp_attribs.attack, player.comp_attribs.health.hp
                    );

                    if player.comp_attribs.health.hp <= 0.0 {
                        info!("Player has died");
                        
                        // note: might want to make this an event 
                        // so it can be broadcast to different systems
                        cmd.entity(player.id).despawn();
                    } else {
                        attack_timer.0.reset();
                    }
                }
            }
        }
    }
}
