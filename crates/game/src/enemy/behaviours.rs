use super::*;

pub const FLOAT_HEIGHT: f32 = 0.5;
pub const IDLE_TO_RUN_TRESHOLD: f32 = 0.01;

pub fn plugin(app: &mut App) {
    // app.add_plugins(());

    app.add_systems(
        Update,
        (
            proximity_emitter.run_if(transform_changed),
            movement_emitter,
            update_target_position,
            move_to,
        ).run_if(in_state(Screen::Gameplay)),
    );

    app.insert_resource(PlayerTargetUpdateTimer(Timer::from_seconds(0.25, TimerMode::Repeating)));

    app.add_event::<ProximityEvent>()
        .add_event::<MovementEvent>();
}

#[derive(Event)]
pub struct ProximityEvent {
    player_entity: Entity,
    enemy_entity: Entity,
}

#[derive(Event)]
pub struct MovementEvent {
    pub entity: Entity,
    pub target: Entity,
}

impl MovementEvent {
    pub fn new(entity: Entity, target: Entity) -> Self {
        Self {
            entity,
            target,
        }
    }
}

#[derive(Resource)]
pub struct PlayerTargetUpdateTimer(Timer);

fn proximity_emitter(
    player_query: Query<(&Transform, Entity), With<Player>>,
    enemy_query: Query<(&Transform, &Enemy, Entity)>,
    mut proximity_evt_writer: EventWriter<ProximityEvent>,
) {
    for (enemy_transform, enemy, enemy_entity) in enemy_query.iter() {
        for (player_transform, player_entity) in player_query.iter() {
            let dist = enemy_transform.translation.distance(player_transform.translation);

            if dist <= enemy.aggro_radius {
                proximity_evt_writer.write(ProximityEvent {
                    player_entity,
                    enemy_entity,
                });
            }
        }
    }
}

fn transform_changed(
    player_query: Query<(), (With<Player>, Changed<GlobalTransform>)>,
    enemy_query: Query<(), (With<Enemy>, Changed<GlobalTransform>)>,
) -> bool {
    !player_query.is_empty() || !enemy_query.is_empty()
}

fn movement_emitter(
    mut proximity_evt_reader: EventReader<ProximityEvent>,
    mut movement_event_writer: EventWriter<MovementEvent>,
) {
    let Some(proximity_evt) = proximity_evt_reader.read().next() else {
        return;
    };

    let move_evt = MovementEvent::new(proximity_evt.enemy_entity, proximity_evt.player_entity);
    movement_event_writer.write(move_evt);
}

fn update_target_position(
    mut movement_evt: EventReader<MovementEvent>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<&mut Enemy, With<Enemy>>,
    mut target_update_timer: ResMut<PlayerTargetUpdateTimer>,
    time: Res<Time>,
) -> Result {
    // target_update_timer.0.tick(time.delta());
    //
    // if !target_update_timer.0.just_finished() {
    //     return Ok(());
    // }

    let Some(movement_evt) = movement_evt.read().next() else {
        return Ok(());
    };

    let Ok(mut enemy) = enemy_query.get_mut(movement_evt.entity) else {
        return Ok(());
    };

    let Ok(player_transform) = player_query.get(movement_evt.target) else {
        return Ok(());
    };

    enemy.target_entity = Some(movement_evt.target);
    enemy.target_position = Some(player_transform.translation);

    Ok(())
}

fn move_to(
    mut enemy_query: Query<(&mut Enemy, &mut TnuaController, &mut StepTimer, &Transform), With<Enemy>>,
    cfg: Res<Config>,
) -> Result {
    for (mut enemy, mut controller, mut step_timer, enemy_transform) in enemy_query.iter_mut() {
        if let Some(target_position) = enemy.target_position {
            let mut direction = target_position - enemy_transform.translation;

            let distance = direction.xz().length();
            if distance <= 0.15 {
                controller.basis(TnuaBuiltinWalk {
                    desired_velocity: Vec3::ZERO,
                    desired_forward: None,
                    ..default()
                });

                enemy.target_position = None;
                return Ok(());
            }

            direction = direction.normalize_or_zero();
            controller.basis(builtin_walk(enemy.speed, direction));

            // update step timer dynamically based on actual speed
            // normal step: 0.475
            // sprint step (x1.5): 0.354
            // step on sprint timer: 0.317
            let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                return Ok(());
            };

            let current_actual_speed = basis_state.running_velocity.length();
            if current_actual_speed > IDLE_TO_RUN_TRESHOLD {
                let ratio = enemy.speed / current_actual_speed;
                let adjusted_step_time_f32 = cfg.timers.step * ratio;
                let adjusted_step_time = Duration::from_secs_f32(adjusted_step_time_f32);
                // info!("step timer:{adjusted_step_time_f32}s");
                step_timer.set_duration(adjusted_step_time);
            }
        }
    }

    Ok(())
}

fn builtin_walk(speed: f32, direction: Vec3) -> TnuaBuiltinWalk {
    TnuaBuiltinWalk {
        float_height: 0.5,
        cling_distance: FLOAT_HEIGHT + 0.01, // Slightly higher than float_height for a bit of "give".
        spring_strength: 500.0,              // Stronger spring for a more grounded feel.
        spring_dampening: 1.0,               // Slightly reduced dampening for a more responsive spring.
        acceleration: 80.0,                  // Increased acceleration for snappier movement starts and stops.
        air_acceleration: 30.0,              // Allow for some air control, but less than ground.
        free_fall_extra_gravity: 70.0,       // Slightly increased for a less floaty fall.
        tilt_offset_angvel: 7.0,             // Increased for a slightly faster righting response.
        tilt_offset_angacl: 700.0,           // Increased acceleration to reach the target righting speed.
        turning_angvel: 12.0,                // Increased for more responsive turning.
        desired_velocity: direction * speed,
        desired_forward: Dir3::new(direction).ok(),
        ..Default::default()
    }
}