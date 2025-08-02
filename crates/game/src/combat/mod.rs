use super::*;
use std::time::Duration;

pub use models::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_attack_event.run_if(in_state(Screen::Gameplay)),
    );

    // app.add_systems(OnEnter(Screen::Gameplay), spawn_enemy);
}

#[derive(Component, Debug)]
pub struct AttackTimer(pub Timer);

fn handle_attack_event(
    mut enemy_query: Query<(&Enemy, &Transform, &mut AttackTimer)>,
    mut player_query: Query<(&mut Player, &Transform)>,
    time: Res<Time>,
) {
    println!("Handle attack");

    // enemy
    for (enemy, enemy_transform, mut attack_timer) in enemy_query.iter_mut() {
        for (mut player, _player_transform) in player_query
            .iter_mut()
            .filter(|(_, player_tfm)| {
                let dist = player_tfm.translation.xy().distance(enemy_transform.translation.xy());
                let is_in_range = dist <= enemy.comp_attribs.attack_range;

                println!("is in range: {}, tfm length: {}", is_in_range, dist);

                is_in_range
            })
        {
            println!("try deal damage to player");
            if attack_timer.0.finished() {
                player.comp_attribs.health.hp -= enemy.comp_attribs.attack;
                attack_timer.0.reset();

                println!("Enemy deals {} damage to Player", enemy.comp_attribs.attack);

                if player.comp_attribs.health.hp < 1 {
                    println!("Player has died");
                }

                attack_timer.0.tick(time.delta());
            }
        }
    }
}
