use super::*;
use bevy_third_person_camera::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(Screen::Title), spawn_skybox_to_camera)
        .add_systems(OnEnter(Screen::Gameplay), spawn_tpv_cam)
        .add_systems(OnExit(Screen::Gameplay), despawn_tpv_cam)
        .add_systems(OnEnter(Screen::Gameplay), insert_log_tpv_cam_timer)
        .add_systems(OnExit(Screen::Gameplay), remove_log_tpv_cam_timer)
        // .add_systems(Update, log_tpv_cam_pos.run_if(in_state(Screen::Gameplay)))
        .add_systems(
            Update,
            (tick_splash_timer.in_set(Set::TickTimers)).run_if(in_state(Screen::Gameplay)),
        )
        .add_observer(toggle_cam_cursor);
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        SceneCamera,
        Camera3d::default(),
        Msaa::Sample4,
        IsDefaultUiCamera,
        Transform::from_xyz(100., 80., 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Transform::from_xyz(40., 40., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            hdr: true,
            ..Default::default()
        },
    ));
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct LogTpvCameraTimer(Timer);

impl Default for LogTpvCameraTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

fn insert_log_tpv_cam_timer(mut commands: Commands) {
    commands.init_resource::<LogTpvCameraTimer>();
}

fn remove_log_tpv_cam_timer(mut commands: Commands) {
    commands.remove_resource::<LogTpvCameraTimer>();
}

fn tick_splash_timer(time: Res<Time>, mut timer: ResMut<LogTpvCameraTimer>) {
    timer.0.tick(time.delta());
}

// fn log_tpv_cam_pos(
//     camera_query: Query<(&Transform, &Camera3d), With<ThirdPersonCamera>>,
//     mut timer: ResMut<LogTpvCameraTimer>,
// ) -> Result {
//     if timer.0.just_finished() {
//         let camera = camera_query.single()?;
//         println!("Camera Transform X: {}, Y: {}, Z: {}", camera.0.translation.x, camera.0.translation.y, camera.0.translation.z);
//         println!("Camera Rotation X: {}, Y: {}, Z: {}", camera.0.rotation.x, camera.0.rotation.y, camera.0.rotation.z);
//         timer.0.reset();
//     }
// 
//     Ok(())
// }

fn spawn_tpv_cam(
    cfg: Res<Config>,
    mut commands: Commands,
    mut camera: Query<Entity, With<SceneCamera>>,
    mut scene_cam: Query<Entity, With<ThirdPersonCamera>>,
) -> Result {
    let camera = camera.single_mut()?;
    if scene_cam.single_mut().is_ok() {
        debug!("ThirdPersonCamera already exist");
        return Ok(());
    }

    commands.entity(camera).insert((
        ThirdPersonCamera {
            // aim_speed: 3.0,     // default
            // aim_zoom: 0.7,      // default
            // aim_enabled: true,

            zoom_enabled: true, // default
            zoom: Zoom::new(cfg.player.zoom.0, cfg.player.zoom.1),

            offset_enabled: true,
            offset_toggle_enabled: true,

            cursor_lock_active: false,
            cursor_lock_toggle_enabled: false,

            gamepad_settings: CustomGamepadSettings::default(),

            // bounds: vec![Bound::NO_FLIP, Bound::ABOVE_FLOOR],
            ..default()
        },
        RigidBody::Kinematic,
        Collider::sphere(1.0),
        Projection::from(PerspectiveProjection {
            fov: cfg.player.fov.to_radians(),
            ..Default::default()
        }),
    ));

    Ok(())
}

fn despawn_tpv_cam(mut commands: Commands, mut camera: Query<Entity, With<SceneCamera>>) {
    if let Ok(camera) = camera.single_mut() {
        commands
            .entity(camera)
            .remove::<RigidBody>()
            .remove::<ThirdPersonCamera>();
    }
}

fn toggle_cam_cursor(_: Trigger<OnCamCursorToggle>, mut cam: Query<&mut ThirdPersonCamera>) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };
    cam.cursor_lock_active = !cam.cursor_lock_active;
}

/// Helper trait to get direction of movement based on camera transform
pub trait MovementDirection {
    fn movement_direction(&self, input: Vec2) -> Vec3;
}

impl MovementDirection for Transform {
    fn movement_direction(&self, input: Vec2) -> Vec3 {
        let forward = self.forward();
        let forward_flat = Vec3::new(forward.x, 0.0, forward.z);
        let right = forward_flat.cross(Vec3::Y).normalize();
        let direction = (right * input.x) + (forward_flat * input.y);
        direction.normalize_or_zero()
    }
}
