use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use std::{f32::consts::FRAC_PI_2, ops::Range};

const MOVEMENT_SPEED: f32 = 5.0;

#[derive(Component)]
struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .add_plugins(MeshPickingPlugin)
            .add_systems(Startup, (setup_player, setup_camera).chain())
            .add_systems(Update, (player_controls, camera_controls).chain());
    }
}

#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            orbit_distance: 10.0,
            pitch_speed: 0.003,
            pitch_range: -FRAC_PI_2 + 0.01..0.0,
            yaw_speed: 0.004,
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct CameraTarget;

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Player,
        Name::new("Player"),
        CameraTarget,
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

    // TODO: Remove when environment is added
    // obstacle
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 1.0, 4.0),
    ));
}

fn player_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Single<&mut Transform, (With<Player>, Without<MainCamera>)>,
    camera: Single<&Transform, (With<MainCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let mut direction: Vec3 = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    } else if keys.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    } else if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let (yaw, _, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let rotation_matrix = Mat3::from_rotation_y(yaw);

    direction = rotation_matrix * direction.normalize_or_zero();
    player.translation += direction * MOVEMENT_SPEED * time.delta_secs();
}

fn setup_camera(
    mut commands: Commands,
    target: Single<&Transform, With<CameraTarget>>,
    camera_settings: Res<CameraSettings>,
) {
    commands.spawn((
        MainCamera,
        Name::new("MainCamera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, camera_settings.orbit_distance)
            .looking_at(target.translation, Vec3::Y),
    ));
}

fn camera_controls(
    camera_settings: Res<CameraSettings>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
    camera_target: Single<&Transform, (With<CameraTarget>, Without<MainCamera>)>,
    mut ray_cast: MeshRayCast,
) {
    let delta = mouse_motion.delta;
    let delta_pitch = -delta.y * camera_settings.pitch_speed;
    let delta_yaw = delta.x * camera_settings.yaw_speed;
    let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );
    let yaw = yaw - delta_yaw;

    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    camera.translation =
        camera_target.translation - camera.forward() * camera_settings.orbit_distance;

    // TODO: Move ray casting to its own system
    let ray_pos = camera_target.translation;
    let ray_dir = -camera.forward().normalize();
    let ray = Ray3d::new(ray_pos, Dir3::new(ray_dir).unwrap());

    let Some((_, hit)) = ray_cast
        .cast_ray(ray, &MeshRayCastSettings::default())
        .first()
    else {
        return;
    };

    if hit.distance < camera_settings.orbit_distance {
        camera.translation =
            camera_target.translation - camera.forward().normalize() * (hit.distance - 0.05);
    }
}
