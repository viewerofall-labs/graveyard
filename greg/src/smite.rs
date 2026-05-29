use bevy::prelude::*;

use crate::camera::CameraShake;
use crate::character::{Character, CharacterKind, WalkState, random_position};
use crate::sfx::{play_sfx, Sfx};
use crate::torment::TormentEvent;
use crate::ui::SpeechQueue;

#[derive(Component)]
pub struct AshParticle {
    pub vel: Vec3,
    pub ang_vel: Vec3,
    pub owner: Entity,
}

#[derive(Component)]
pub struct LightningBolt {
    pub lifetime: f32,
    pub owner: Entity,
}

#[derive(Resource, Default)]
pub struct SmiteRequests(pub Vec<(Entity, Vec3)>);

pub fn smite_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut chars: Query<(Entity, &Transform, &mut Visibility, &mut WalkState, &Character)>,
    mut shake: ResMut<CameraShake>,
    mut requests: ResMut<SmiteRequests>,
    mut speech: ResMut<SpeechQueue>,
    mut torment_ev: EventWriter<TormentEvent>,
) {
    let triggers = [
        (KeyCode::KeyD, CharacterKind::Greg),
        (KeyCode::KeyC, CharacterKind::Fred),
    ];
    for (key, kind) in triggers {
        if !keys.just_pressed(key) {
            continue;
        }
        for (e, t, mut vis, mut walk, ch) in chars.iter_mut() {
            if ch.kind != kind {
                continue;
            }
            if walk.smited
                || walk.dismembered
                || walk.dying_timer > 0.0
                || walk.ascending_remaining > 0.0
            {
                continue;
            }
            trigger_smite(e, t.translation, &mut walk, &mut vis, &mut shake, &mut requests);
            if matches!(ch.kind, CharacterKind::Greg) {
                torment_ev.send(TormentEvent::Smite);
            }
            let (line, color) = match ch.kind {
                CharacterKind::Greg => ("AAA-", [0.95, 0.95, 0.95]),
                CharacterKind::Fred => ("noooo-", [1.0, 0.3, 0.3]),
            };
            speech.0.push((e, line.into(), color));
        }
    }
}

pub fn trigger_smite(
    e: Entity,
    pos: Vec3,
    walk: &mut WalkState,
    vis: &mut Visibility,
    shake: &mut CameraShake,
    requests: &mut SmiteRequests,
) {
    walk.smited = true;
    walk.smite_timer = 2.0;
    walk.held = false;
    walk.is_staring = false;
    walk.fallen_remaining = 0.0;
    walk.bounce_remaining = 0.0;
    walk.in_pit = 0.0;
    walk.jailed_remaining = 0.0;
    *vis = Visibility::Hidden;
    shake.intensity = shake.intensity.max(0.8);
    requests.0.push((e, pos));
}

pub fn spawn_smite_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut requests: ResMut<SmiteRequests>,
    sfx: Res<Sfx>,
) {
    for (owner, pos) in requests.0.drain(..) {
        let bolt_mesh = meshes.add(Cuboid::new(0.3, 14.0, 0.3));
        let bolt_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 1.0, 1.0),
            emissive: LinearRgba::rgb(8.0, 12.0, 18.0),
            unlit: true,
            ..default()
        });
        commands.spawn((
            Mesh3d(bolt_mesh),
            MeshMaterial3d(bolt_mat),
            Transform::from_xyz(pos.x, 7.0, pos.z),
            LightningBolt { lifetime: 0.22, owner },
        ));

        let ash_mesh = meshes.add(Cuboid::new(0.09, 0.09, 0.09));
        let ash_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.09, 0.09),
            perceptual_roughness: 1.0,
            ..default()
        });
        for _ in 0..45 {
            let offset = Vec3::new(
                (rand::random::<f32>() - 0.5) * 0.45,
                rand::random::<f32>() * 1.6,
                (rand::random::<f32>() - 0.5) * 0.45,
            );
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = 1.5 + rand::random::<f32>() * 3.5;
            let vel = Vec3::new(
                angle.cos() * speed,
                2.5 + rand::random::<f32>() * 3.5,
                angle.sin() * speed,
            );
            let ang_vel = Vec3::new(
                (rand::random::<f32>() - 0.5) * 12.0,
                (rand::random::<f32>() - 0.5) * 12.0,
                (rand::random::<f32>() - 0.5) * 12.0,
            );
            commands.spawn((
                Mesh3d(ash_mesh.clone()),
                MeshMaterial3d(ash_mat.clone()),
                Transform::from_translation(pos + offset),
                AshParticle { vel, ang_vel, owner },
            ));
        }

        play_sfx(&mut commands, &sfx.smite);
    }
}

pub fn update_smite(
    mut commands: Commands,
    mut chars: Query<(Entity, &mut Transform, &mut Visibility, &mut WalkState, &Character)>,
    particles: Query<(Entity, &AshParticle)>,
    bolts: Query<(Entity, &LightningBolt)>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    let mut finished: Vec<Entity> = Vec::new();
    for (e, mut t, mut vis, mut walk, _) in chars.iter_mut() {
        if !walk.smited {
            continue;
        }
        walk.smite_timer -= dt;
        if walk.smite_timer <= 0.0 {
            walk.smited = false;
            walk.smite_timer = 0.0;
            walk.anger = 1.0;
            walk.hp = 1.0;
            walk.bruise = 0.0;
            walk.target = random_position();
            walk.target_timer = 0.4;
            t.translation = random_position();
            t.rotation = Quat::IDENTITY;
            *vis = Visibility::Inherited;
            finished.push(e);
        }
    }
    for owner in finished {
        for (e, p) in particles.iter() {
            if p.owner == owner {
                commands.entity(e).despawn();
            }
        }
        for (e, b) in bolts.iter() {
            if b.owner == owner {
                commands.entity(e).despawn();
            }
        }
    }
}

pub fn update_lightning(
    mut commands: Commands,
    mut q: Query<(Entity, &mut LightningBolt, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (e, mut bolt, mut t) in q.iter_mut() {
        bolt.lifetime -= dt;
        if bolt.lifetime <= 0.0 {
            commands.entity(e).despawn();
        } else {
            let f = 0.7 + rand::random::<f32>() * 0.5;
            t.scale = Vec3::new(f, 1.0, f);
        }
    }
}

pub fn update_ash(mut q: Query<(&mut Transform, &mut AshParticle)>, time: Res<Time>) {
    let dt = time.delta().as_secs_f32();
    for (mut t, mut p) in q.iter_mut() {
        if t.translation.y <= 0.05 && p.vel.length_squared() < 0.001 {
            continue;
        }
        p.vel.y -= 14.0 * dt;
        t.translation += p.vel * dt;
        let spin = Quat::from_euler(
            EulerRot::XYZ,
            p.ang_vel.x * dt,
            p.ang_vel.y * dt,
            p.ang_vel.z * dt,
        );
        t.rotation = (spin * t.rotation).normalize();
        if t.translation.y <= 0.05 {
            t.translation.y = 0.05;
            p.vel = Vec3::ZERO;
            p.ang_vel = Vec3::ZERO;
        }
    }
}
