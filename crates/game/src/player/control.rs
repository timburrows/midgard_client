use super::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinCrouch, TnuaBuiltinDash},
    control_helpers::TnuaSimpleAirActionsCounter,
};
use event::types::{AttackEvent, EnemyClickEvent, GroundClickEvent};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            movement.in_set(TnuaUserControlsSystemSet),
            attack_enemy
        ).run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(handle_sprint_in)
        .add_observer(handle_sprint_out)
        .add_observer(handle_jump)
        .add_observer(handle_dash)
        .add_observer(crouch_in)
        .add_observer(crouch_out);
}

fn movement(
    cfg: Res<Config>,
    actions: Single<&Actions<GameplayCtx>>,
    mut ground_click_evt: EventReader<GroundClickEvent>,
    mut player_query: Query<(&mut Player, &mut TnuaController, &mut StepTimer, &Transform)>,
) -> Result {
    let actions = actions.into_inner();

    let mut desired_velocity = Vec3::ZERO;
    let mut desired_forward: Option<Dir3> = None;

    let Ok((mut player, mut controller, mut step_timer, transform)) = player_query.single_mut()
    else {
        error!("Player not found");
        return Ok(());
    };

    if let Some(click) = ground_click_evt.read().next() {
        player.target_position = Some(click.position);
    };

    if let Some(target_position) = player.target_position {
        let mut direction = target_position - transform.translation;
        let distance = direction.xz().length();
        direction = direction.normalize_or_zero();

        desired_velocity = direction * player.speed;
        desired_forward = Dir3::new(direction).ok();

        if distance <= 0.05 {
            player.target_position = None;
        }
    };

    controller.basis(TnuaBuiltinWalk {
        float_height: FLOAT_HEIGHT,
        cling_distance: FLOAT_HEIGHT + 0.01, // Slightly higher than float_height for a bit of "give".
        spring_strength: 500.0,              // Stronger spring for a more grounded feel.
        spring_dampening: 1.0, // Slightly reduced dampening for a more responsive spring.
        acceleration: 80.0,    // Increased acceleration for snappier movement starts and stops.
        air_acceleration: 30.0, // Allow for some air control, but less than ground.
        free_fall_extra_gravity: 70.0, // Slightly increased for a less floaty fall.
        tilt_offset_angvel: 7.0, // Increased for a slightly faster righting response.
        tilt_offset_angacl: 700.0, // Increased acceleration to reach the target righting speed.
        turning_angvel: 12.0,  // Increased for more responsive turning.

        desired_velocity,
        desired_forward,

        ..default()
    });

    // Check if crouch is currently active and apply TnuaBuiltinCrouch as an action
    if actions.value::<Crouch>()?.as_bool() {
        controller.action(TnuaBuiltinCrouch {
            float_offset: 0.0,
            height_change_impulse_for_duration: 0.1,
            height_change_impulse_limit: 80.0,
            uncancellable: false,
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
        let ratio = cfg.player.movement.speed / current_actual_speed;
        let adjusted_step_time_f32 = cfg.timers.step * ratio;
        let adjusted_step_time = Duration::from_secs_f32(adjusted_step_time_f32);
        // info!("step timer:{adjusted_step_time_f32}s");
        step_timer.set_duration(adjusted_step_time);
    }

    Ok(())
}

fn handle_sprint_in(
    on: Trigger<Started<Sprint>>,
    cfg: Res<Config>,
    mut player_query: Query<&mut Player, With<GameplayCtx>>,
) -> Result {
    let entity = on.target();
    if let Ok(mut player) = player_query.get_mut(entity) {
        if player.speed <= cfg.player.movement.speed {
            player.speed *= cfg.player.movement.sprint_factor;
        }
    }

    Ok(())
}

fn handle_sprint_out(
    on: Trigger<Completed<Navigate>>,
    cfg: Res<Config>,
    mut player_query: Query<&mut Player, With<GameplayCtx>>,
) {
    let entity = on.target();
    if let Ok(mut player) = player_query.get_mut(entity)
        && player.speed > cfg.player.movement.speed
    {
        player.speed = cfg.player.movement.speed;
    }
}

fn handle_jump(
    on: Trigger<Fired<Jump>>,
    // cfg: Res<Config>,
    // time: Res<Time>,
    mut player_query: Query<
        (
            &mut TnuaController,
            &mut TnuaSimpleAirActionsCounter,
            &mut JumpTimer,
        ),
        With<Player>,
    >,
) -> Result {
    let (mut controller, mut air_counter, mut _jump_timer) = player_query.get_mut(on.target())?;

    // if jump_timer.tick(time.delta()).just_finished() {
    air_counter.update(controller.as_mut()); // Update air counter
    controller.action(TnuaBuiltinJump {
        height: 3.5,
        takeoff_extra_gravity: 50.0, // Increased for a snappier, more immediate lift-off.
        fall_extra_gravity: 40.0,    // To make falling feel more impactful and less floaty.
        shorten_extra_gravity: 80.0, // Increased to allow for very short hops when tapping the jump button.
        peak_prevention_at_upward_velocity: 0.5, // Slightly lower to start applying peak prevention sooner.
        peak_prevention_extra_gravity: 30.0, // Increased to reduce "floatiness" at the jump's apex.
        reschedule_cooldown: Some(0.1), // Allows for a slight "jump buffering" if the button is pressed just before landing.
        disable_force_forward_after_peak: true,
        allow_in_air: true,
        ..Default::default()
    });
    // }

    Ok(())
}

fn handle_dash(
    on: Trigger<Started<Dash>>,
    cfg: Res<Config>,
    actions: Single<&Actions<GameplayCtx>>,
    camera: Query<&Transform, With<SceneCamera>>,
    mut player_query: Query<(&mut TnuaController, &TnuaSimpleAirActionsCounter)>,
) -> Result {
    let (mut controller, air_counter) = player_query.get_mut(on.target())?;
    let cam_transform = camera.single()?;
    let navigate = actions.value::<Navigate>()?.as_axis2d();
    let direction = cam_transform.movement_direction(navigate);

    controller.action(TnuaBuiltinDash {
        speed: 50.,
        displacement: direction * cfg.player.movement.dash_distance,
        desired_forward: Dir3::new(direction).ok(),
        allow_in_air: air_counter.air_count_for(TnuaBuiltinDash::NAME)
            <= cfg.player.movement.actions_in_air.into(),
        ..Default::default()
    });

    Ok(())
}

pub fn crouch_in(
    on: Trigger<Started<Crouch>>,
    cfg: Res<Config>,
    mut player: Query<&mut Player, With<GameplayCtx>>,
    mut tnua: Query<(&mut TnuaAvian3dSensorShape, &mut Collider), With<Player>>,
) -> Result {
    let (mut avian_sensor, mut collider) = tnua.single_mut()?;
    let mut player = player.get_mut(on.target())?;

    collider.set_scale(Vec3::new(1.0, 0.5, 1.0), 4);
    avian_sensor.0.set_scale(Vec3::new(1.0, 0.5, 1.0), 4);
    player.speed *= cfg.player.movement.crouch_factor;

    Ok(())
}

pub fn crouch_out(
    on: Trigger<Completed<Crouch>>,
    cfg: Res<Config>,
    mut player: Query<&mut Player, With<GameplayCtx>>,
    mut tnua: Query<
        (&mut TnuaAvian3dSensorShape, &mut Collider),
        (With<Player>, Without<SceneCamera>),
    >,
) -> Result {
    let (mut avian_sensor, mut collider) = tnua.get_mut(on.target())?;
    let mut player = player.get_mut(on.target())?;

    collider.set_scale(Vec3::ONE, 4);
    avian_sensor.0.set_scale(Vec3::ONE, 4);
    player.speed = cfg.player.movement.speed;

    Ok(())
}

fn attack_enemy(
    mut enemy_click_event: EventReader<EnemyClickEvent>,
    mut attack_event: EventWriter<AttackEvent>,
    enemy_query: Query<(&Enemy, &Transform, &Collider)>,
    player_query: Query<(&Player, &ComputedAttributes, &Transform, &Collider)>,
) -> Result {
    const COLLISION_BUFFER: f32 = 0.5;

    for enemy_click_evt in enemy_click_event.read() {
        let Ok((enemy, enemy_transform, enemy_collider)) = enemy_query.get(enemy_click_evt.target)
        else {
            warn!("Invalid Enemy or no longer exists");
            continue;
        };

        let Ok((player, player_comp_attribs, player_transform, player_collider)) =
            player_query.get(enemy_click_evt.player)
        else {
            warn!("Invalid Player or no longer exists");
            continue;
        };

        let Some(player_radius) = utils::get_capsule_radius(player_collider) else {
            error!("Player has no capsule");
            continue;
        };

        let Some(enemy_radius) = utils::get_capsule_radius(enemy_collider) else {
            error!("Enemy has no capsule");
            continue;
        };

        let direction = player_transform.translation - enemy_transform.translation;
        let dist = direction.xz().length();

        let collider_dist = enemy_radius + player_radius + COLLISION_BUFFER;
        let is_in_range = dist <= collider_dist + player_comp_attribs.attack_range;

        if is_in_range {
            attack_event.write(AttackEvent {
                attacker: player.id,
                target: enemy.id,
            });
        }
    }

    Ok(())
}
