use super::*;
use event::types::*;

pub fn plugin(app: &mut App) {
    app.add_observer(enemy_spawned)
        .add_observer(player_spawned);
}

fn enemy_click_emitter(
    click_event: Trigger<Pointer<Click>>,
    enemy_query: Query<&Enemy>,
    player_query: Query<&Player>,
    mut enemy_click_events: EventWriter<EnemyClickEvent>,
) {
    let target_entity = click_event.target();

    // fixme: this will break when there are multiple players on screen
    let Ok(player_entity) = player_query.single() else {
        warn!("No player found");
        return;
    };

    if let Ok(enemy) = enemy_query.get(target_entity) {
        enemy_click_events.write(EnemyClickEvent {
            target: enemy.id,
            player: player_entity.id,
        });
    } else {
        warn!("Clicked entity is not an Enemy: {:?}", target_entity);
    }
}

fn enemy_spawned(on: Trigger<OnAdd, Enemy>, mut enemies: Query<&mut Enemy>, mut cmd: Commands) {
    let enemy = on.target();
    if let Ok(mut e) = enemies.get_mut(enemy) {
        e.id = enemy;
    }

    cmd.entity(enemy).observe(enemy_click_emitter);
}

fn player_spawned(
    on: Trigger<OnAdd, Player>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    let player = on.target();
    if let Ok(mut p) = players.get_mut(player) {
        p.id = player;
    }
    commands.trigger(SwitchInputCtx::new(player, Context::Gameplay));
    commands.trigger(SwitchInputCtx::from_context(Context::Gameplay));
}