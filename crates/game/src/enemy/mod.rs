use super::*;
use bevy::scene::SceneInstanceReady;
use std::{f32::consts::PI, time::Duration};
mod animation;
mod control;

pub use animation::*;
pub use control::*;

pub const IDLE_TO_RUN_TRESHOLD: f32 = 0.01;
pub const FLOAT_HEIGHT: f32 = 0.5;

pub fn plugin(app: &mut App) {
    app.add_plugins((control::plugin));

    // app.add_systems(OnEnter(Screen::Gameplay), spawn_enemy);
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
    let pos = Transform::from_translation(Vec3::new(15.0, 20.0, 0.0)).with_rotation(enemy_rot);
    let enemy = Enemy {
        id: Entity::PLACEHOLDER,
        speed: cfg.player.movement.speed,
        animation_state: EnemyAnimationState::StandIdle,
        ..default()
    };

    let collider = Collider::capsule(cfg.player.hitbox.radius, cfg.player.hitbox.height);

    commands
        .spawn((
            StateScoped(Screen::Gameplay),
            pos,
            enemy,
            // input context
            (
                GameplayCtx,
                CurrentCtx(Context::Gameplay),
                Actions::<GameplayCtx>::default(),
            ),
            // physics
            (
                collider,
                RigidBody::Dynamic,
                // Friction::ZERO.with_combine_rule(CoefficientCombine::Multiply),
            ),
            JumpTimer(Timer::from_seconds(cfg.timers.jump, TimerMode::Repeating)),
            StepTimer(Timer::from_seconds(cfg.timers.step, TimerMode::Repeating)),
            InheritedVisibility::default(), // silence the warning because of adding SceneRoot as a child
        ))
        // spawn character mesh as child to adjust mesh position relative to the player origin
        .with_children(|parent| {
            let mut e = parent.spawn((Transform::from_xyz(0.0, -1.5, 0.0), mesh));

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
