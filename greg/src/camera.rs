use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
}

#[derive(Resource, Default)]
pub struct CameraShake {
    pub intensity: f32,
}

pub fn orbit_camera(
    mut query: Query<(&mut Transform, &mut OrbitCamera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut motion_events: EventReader<MouseMotion>,
    mut wheel_events: EventReader<MouseWheel>,
    mut shake: ResMut<CameraShake>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut orbit)) = query.get_single_mut() else {
        motion_events.clear();
        wheel_events.clear();
        return;
    };

    let mut delta = Vec2::ZERO;
    if mouse_buttons.pressed(MouseButton::Right) {
        for ev in motion_events.read() {
            delta += ev.delta;
        }
    } else {
        motion_events.clear();
    }

    let mut scroll = 0.0;
    for ev in wheel_events.read() {
        scroll += ev.y;
    }

    if delta != Vec2::ZERO {
        orbit.yaw -= delta.x * 0.005;
        orbit.pitch = (orbit.pitch - delta.y * 0.005)
            .clamp(0.1, std::f32::consts::FRAC_PI_2 - 0.05);
    }
    if scroll != 0.0 {
        orbit.radius = (orbit.radius - scroll * 0.8).clamp(4.0, 40.0);
    }

    let pos = Vec3::new(
        orbit.radius * orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.radius * orbit.pitch.sin(),
        orbit.radius * orbit.yaw.cos() * orbit.pitch.cos(),
    );

    let shake_offset = if shake.intensity > 0.001 {
        Vec3::new(
            (rand::random::<f32>() - 0.5) * 2.0,
            (rand::random::<f32>() - 0.5) * 2.0,
            (rand::random::<f32>() - 0.5) * 2.0,
        ) * shake.intensity
    } else {
        Vec3::ZERO
    };

    *transform = Transform::from_translation(pos + shake_offset).looking_at(Vec3::ZERO, Vec3::Y);

    let dt = time.delta().as_secs_f32();
    shake.intensity = (shake.intensity - dt * 4.0).max(0.0);
}

pub fn ray_sphere(ray: Ray3d, center: Vec3, radius: f32) -> Option<f32> {
    let dir = *ray.direction;
    let oc = ray.origin - center;
    let b = oc.dot(dir);
    let c = oc.length_squared() - radius * radius;
    let d = b * b - c;
    if d < 0.0 {
        return None;
    }
    let t = -b - d.sqrt();
    if t < 0.0 {
        None
    } else {
        Some(t)
    }
}

pub fn ray_plane_y(ray: Ray3d, y: f32) -> Option<Vec3> {
    let dir = *ray.direction;
    if dir.y.abs() < 1e-6 {
        return None;
    }
    let t = (y - ray.origin.y) / dir.y;
    if t < 0.0 {
        None
    } else {
        Some(ray.origin + dir * t)
    }
}
