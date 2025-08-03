use super::*;

pub const FLOAT_HEIGHT: f32 = 0.5;
pub const IDLE_TO_RUN_TRESHOLD: f32 = 0.01;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            proximity_emitter.run_if(transform_changed),
            movement_emitter,
            update_target_position,
            move_to_pos,
            attack_player,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

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
        Self { entity, target }
    }
}

fn proximity_emitter(
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
) -> Result {
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

fn move_to_pos(
    mut enemy_query: Query<
        (
            &mut Enemy,
            &ComputedAttributes,
            &mut TnuaController,
            &mut StepTimer,
            &Transform,
            &Collider,
        ),
        With<Enemy>,
    >,
    player_query: Query<&Collider, With<Player>>,
    cfg: Res<Config>,
) -> Result {
    let mut desired_velocity = Vec3::ZERO;
    let mut desired_forward: Option<Dir3> = None;

    for (
        enemy,
        enemy_comp_attribs,
        mut controller,
        mut step_timer,
        enemy_transform,
        enemy_collider,
    ) in enemy_query.iter_mut()
    {
        if let Some(target_position) = enemy.target_position {
            let mut direction = target_position - enemy_transform.translation;
            let dist = direction.xz().length();
            direction = direction.normalize_or_zero();

            let player_collider = player_query.single()?;
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

            if dist >= collision_dist {
                desired_velocity = direction * enemy_comp_attribs.move_speed;
                desired_forward = Dir3::new(direction).ok();
            }

            controller.basis(TnuaBuiltinWalk {
                float_height: FLOAT_HEIGHT,
                cling_distance: FLOAT_HEIGHT + 0.01, // Slightly higher than float_height for a bit of "give".
                spring_strength: 500.0,              // Stronger spring for a more grounded feel.
                spring_dampening: 1.0, // Slightly reduced dampening for a more responsive spring.
                acceleration: 80.0, // Increased acceleration for snappier movement starts and stops.
                air_acceleration: 30.0, // Allow for some air control, but less than ground.
                free_fall_extra_gravity: 70.0, // Slightly increased for a less floaty fall.
                tilt_offset_angvel: 7.0, // Increased for a slightly faster righting response.
                tilt_offset_angacl: 700.0, // Increased acceleration to reach the target righting speed.
                turning_angvel: 12.0,      // Increased for more responsive turning.

                desired_velocity,
                desired_forward,

                ..default()
            });
        }

        // update step timer dynamically based on actual speed
        // normal step: 0.475
        // sprint step (x1.5): 0.354
        // step on sprint timer: 0.317
        let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
            return Ok(());
        };

        let current_actual_speed = basis_state.running_velocity.length();
        if current_actual_speed > IDLE_TO_RUN_TRESHOLD {
            let ratio = enemy_comp_attribs.move_speed / current_actual_speed;
            let adjusted_step_time_f32 = cfg.timers.step * ratio;
            let adjusted_step_time = Duration::from_secs_f32(adjusted_step_time_f32);
            // info!("step timer:{adjusted_step_time_f32}s");
            step_timer.set_duration(adjusted_step_time);
        }
    }

    Ok(())
}

fn attack_player(
    mut attack_event: EventWriter<AttackEvent>,
    mut enemy_query: Query<(&Enemy, &ComputedAttributes, &Transform, &Collider)>,
    player_query: Query<(&Player, &Transform, &Collider)>,
) -> Result {
    let (player, player_transform, player_collider) = player_query.single()?;

    for (enemy, enemy_comp_attribs, enemy_transform, enemy_collider) in enemy_query.iter_mut() {
        let direction = player_transform.translation - enemy_transform.translation;
        let dist = direction.xz().length();

        let player_radius = get_capsule_radius(player_collider);
        let enemy_radius = get_capsule_radius(enemy_collider);

        // note: we do this same check in the combat module to prohibit out-of-range attacks
        // however, later logic will process attack decisions based on multiple ranges (e.g spells, skills)
        const COLLISION_BUFFER: f32 = 0.5;
        let collider_dist = enemy_radius + player_radius + COLLISION_BUFFER;
        let is_in_range = dist <= collider_dist + enemy_comp_attribs.attack_range;

        if is_in_range {
            attack_event.write(AttackEvent {
                attacker: enemy.id,
                target: player.id,
            });
        }
    }

    Ok(())
}