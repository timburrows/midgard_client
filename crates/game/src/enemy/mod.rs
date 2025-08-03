use super::*;
use bevy_tnua_avian3d::{TnuaAvian3dSensorShape};
use std::{f32::consts::PI, time::Duration};

mod animation;
mod behaviours;

pub use animation::*;
pub use combat::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((behaviours::plugin,));
    app.add_systems(OnEnter(Screen::Gameplay), spawn_enemy);
}

pub fn spawn_enemy(
    cfg: Res<Config>,
    models: Res<Models>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    let Some(gltf) = gltf_assets.get(&models.enemy) else {
        return Ok(());
    };

    let enemy_rot = Quat::from_rotation_y(PI);
    let mesh = SceneRoot(gltf.scenes[0].clone());
    let pos = Transform::from_translation(Vec3::new(20.0, 5.0, 0.0)).with_rotation(enemy_rot);
    let enemy = Enemy {
        id: Entity::PLACEHOLDER,
        // animation_state: AnimationState::StandIdle,
        attributes: Attributes::default(),
        comp_attribs: ComputedAttributes {
            move_speed: 3.0,
            attack: 2.0,
            attack_range: 2.0,
            attack_rate: 1.6,
            health: Health::new(10.0),
            ..default()
        },
        ..default()
    };

    let collider = Collider::capsule(cfg.player.hitbox.radius, cfg.player.hitbox.height);
    let attack_timer = AttackRateTimer::new(enemy.comp_attribs.attack_rate);

    commands
        .spawn((
            StateScoped(Screen::Gameplay),
            pos,
            enemy.clone(),
            (
                TnuaController::default(),
                // Tnua can fix the rotation, but the character will still get rotated before it can do so.
                // By locking the rotation we can prevent this.
                LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
                // TnuaAnimatingState::<AnimationState>::default(),
                // TnuaSimpleAirActionsCounter::default(),
                // A sensor shape is not strictly necessary, but without it we'll get weird results.
                TnuaAvian3dSensorShape(collider.clone()),
            ),
            // physics
            (
                collider,
                RigidBody::Dynamic,
                // Friction::ZERO.with_combine_rule(CoefficientCombine::Multiply),
            ),
            // Timers
            StepTimer(Timer::from_seconds(cfg.timers.step, TimerMode::Repeating)),
            attack_timer,
            InheritedVisibility::default(), // silence the warning because of adding SceneRoot as a child
        ))
        // spawn character mesh as child to adjust mesh position relative to the player origin
        .with_children(|parent| {
            let mut e = parent.spawn((Transform::from_xyz(0.0, -1.5, 0.0), mesh));
            // e.observe(prepare_animations);

            // DEBUG
            // let collider_mesh = Mesh::from(Capsule3d::new(
            //     cfg.player.hitbox.radius,
            //     cfg.player.hitbox.height,
            // ));
            // let debug_collider_mesh = Mesh3d(meshes.add(collider_mesh.clone()));
            // let debug_collider_color: MeshMaterial3d<StandardMaterial> =
            //     MeshMaterial3d(materials.add(Color::srgba(0.9, 0.9, 0.9, 0.1)));
            // parent.spawn((
            //     debug_collider_mesh,
            //     debug_collider_color,
            //     Transform::from_xyz(0.0, -0.1, 0.0),
            // ));
            // DEBUG
        });

    Ok(())
}
