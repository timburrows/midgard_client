use super::*;
use event::types::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            proximity_emitter.run_if(transform_changed),
            movement_emitter,
            ground_click_emitter,
        ).run_if(in_state(Screen::Gameplay)),
    );
}

pub fn proximity_emitter(
    player_query: Query<(&Transform, Entity), With<Player>>,
    enemy_query: Query<(&Transform, &Enemy, Entity)>,
    mut proximity_evt_writer: EventWriter<ProximityEvent>,
) {
    for (enemy_transform, enemy, enemy_entity) in enemy_query.iter() {
        for (player_transform, player_entity) in player_query.iter() {
            let dist = enemy_transform
                .translation
                .xy()
                .distance(player_transform.translation.xy());

            if dist <= enemy.aggro_radius {
                proximity_evt_writer.write(ProximityEvent {
                    player_entity,
                    enemy_entity,
                });
            }
        }
    }
}

fn movement_emitter(
    mut proximity_evt_reader: EventReader<ProximityEvent>,
    mut movement_event_writer: EventWriter<PositionChangeEvent>,
) {
    let Some(proximity_evt) = proximity_evt_reader.read().next() else {
        return;
    };

    let move_evt = PositionChangeEvent::new(proximity_evt.enemy_entity, proximity_evt.player_entity);
    movement_event_writer.write(move_evt);
}

fn transform_changed(
    player_query: Query<(), (With<Player>, Changed<GlobalTransform>)>,
    enemy_query: Query<(), (With<Enemy>, Changed<GlobalTransform>)>,
) -> bool {
    !player_query.is_empty() || !enemy_query.is_empty()
}

pub fn ground_click_emitter(
    mut click_events: EventReader<Pointer<Click>>,
    ground_query: Query<&Transform, With<Ground>>,
    mut ground_click_events: EventWriter<GroundClickEvent>,
) {
    for click in click_events.read() {
        if let Ok(transform) = ground_query.get(click.target) {
            let Some(click_position) = click.hit.position else {
                continue;
            };

            let surface_y = transform.translation.y + (transform.scale.y / 2.0);
            let position = Vec3::new(click_position.x, surface_y, click_position.z);

            ground_click_events.write(GroundClickEvent { position });
        }
    }
}
