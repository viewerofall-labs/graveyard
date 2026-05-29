use bevy::prelude::*;

use crate::camera::CameraShake;
use crate::character::{Character, CharacterKind, WalkState, PUNCH_DUR};
use crate::sfx::{play_sfx, Sfx};
use crate::torment::TormentEvent;
use crate::ui::SpeechQueue;

const JAIL_DUR: f32 = 1.5;

#[derive(Component, Clone, Copy)]
pub enum Limb {
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

#[allow(dead_code)]
#[derive(Component, Default, Clone)]
pub enum LimbState {
    #[default]
    Attached,
    Falling {
        vel: Vec3,
        ang_vel: Vec3,
    },
    Lying {
        timer: f32,
    },
    Returning {
        elapsed: f32,
        start_pos: Vec3,
        start_rot: Quat,
    },
}

#[derive(Component)]
pub struct HomeTransform(pub Transform);

#[derive(Component)]
pub struct LimbOwner(pub Entity);

#[derive(Component)]
pub struct JailCell {
    pub owner: Entity,
}

pub fn jail_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut chars: Query<(Entity, &mut WalkState, &Character)>,
    mut shake: ResMut<CameraShake>,
    mut speech: ResMut<SpeechQueue>,
    mut torment_ev: EventWriter<TormentEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<Sfx>,
) {
    let triggers = [
        (KeyCode::KeyJ, CharacterKind::Greg),
        (KeyCode::KeyK, CharacterKind::Fred),
    ];
    for (key, kind) in triggers {
        if !keys.pressed(key) {
            continue;
        }
        for (char_e, mut walk, ch) in chars.iter_mut() {
            if ch.kind != kind {
                continue;
            }
            if walk.smited || walk.dying_timer > 0.0 || walk.ascending_remaining > 0.0 {
                continue;
            }
            let already_jailed = walk.jailed_remaining > 0.0;
            walk.jailed_remaining = JAIL_DUR;

            if !already_jailed {
                walk.held = false;
                walk.is_staring = false;
                walk.fallen_remaining = 0.0;
                walk.bounce_remaining = 0.0;
                shake.intensity = shake.intensity.max(0.25);
                if matches!(ch.kind, CharacterKind::Greg) {
                    torment_ev.send(TormentEvent::Jail);
                }
                play_sfx(&mut commands, &sfx.dismember);
                let (line, color) = match ch.kind {
                    CharacterKind::Greg => ("im in jail", [0.95, 0.95, 0.95]),
                    CharacterKind::Fred => ("THIS IS UNFAIR", [1.0, 0.3, 0.3]),
                };
                speech.0.push((char_e, line.into(), color));
                spawn_jail_cell(&mut commands, &mut meshes, &mut materials, char_e);
            }
        }
    }
}

fn spawn_jail_cell(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    owner: Entity,
) {
    let bar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.28, 0.32),
        emissive: LinearRgba::rgb(0.05, 0.08, 0.18),
        perceptual_roughness: 0.35,
        metallic: 0.85,
        ..default()
    });
    let post_mesh = meshes.add(Cuboid::new(0.08, 2.2, 0.08));
    let frame_x = meshes.add(Cuboid::new(1.6, 0.08, 0.08));
    let frame_z = meshes.add(Cuboid::new(0.08, 0.08, 1.6));
    let vert_bar = meshes.add(Cuboid::new(0.05, 2.0, 0.05));

    let cell = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            JailCell { owner },
        ))
        .id();
    commands.entity(owner).add_child(cell);

    commands.entity(cell).with_children(|c| {
        // 4 corner posts
        for x in [-0.8_f32, 0.8] {
            for z in [-0.8_f32, 0.8] {
                c.spawn((
                    Mesh3d(post_mesh.clone()),
                    MeshMaterial3d(bar_mat.clone()),
                    Transform::from_xyz(x, 0.0, z),
                ));
            }
        }
        // top & bottom frames
        for y in [1.1_f32, -1.1] {
            for z in [-0.8_f32, 0.8] {
                c.spawn((
                    Mesh3d(frame_x.clone()),
                    MeshMaterial3d(bar_mat.clone()),
                    Transform::from_xyz(0.0, y, z),
                ));
            }
            for x in [-0.8_f32, 0.8] {
                c.spawn((
                    Mesh3d(frame_z.clone()),
                    MeshMaterial3d(bar_mat.clone()),
                    Transform::from_xyz(x, y, 0.0),
                ));
            }
        }
        // vertical bars on all 4 sides
        for x in [-0.4_f32, 0.0, 0.4] {
            for z in [-0.8_f32, 0.8] {
                c.spawn((
                    Mesh3d(vert_bar.clone()),
                    MeshMaterial3d(bar_mat.clone()),
                    Transform::from_xyz(x, 0.0, z),
                ));
            }
        }
        for z in [-0.4_f32, 0.0, 0.4] {
            for x in [-0.8_f32, 0.8] {
                c.spawn((
                    Mesh3d(vert_bar.clone()),
                    MeshMaterial3d(bar_mat.clone()),
                    Transform::from_xyz(x, 0.0, z),
                ));
            }
        }
    });
}

pub fn update_jail_cell(
    mut commands: Commands,
    mut chars: Query<&mut WalkState, With<Character>>,
    cells: Query<(Entity, &JailCell)>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for mut walk in chars.iter_mut() {
        if walk.jailed_remaining > 0.0 {
            walk.jailed_remaining = (walk.jailed_remaining - dt).max(0.0);
        }
    }
    for (e, cell) in cells.iter() {
        let should_despawn = match chars.get(cell.owner) {
            Ok(w) => w.jailed_remaining <= 0.0,
            Err(_) => true,
        };
        if should_despawn {
            commands.entity(e).despawn_recursive();
        }
    }
}

pub fn update_dismembered_limbs(
    mut commands: Commands,
    mut limbs: Query<(Entity, &mut Transform, &mut LimbState, &HomeTransform, &LimbOwner)>,
    mut chars: Query<(Entity, &GlobalTransform, &mut WalkState), With<Character>>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (char_e, char_gt, mut walk) in chars.iter_mut() {
        if !walk.dismembered {
            continue;
        }
        let mut all_attached = true;
        for (lim_e, mut tf, mut state, home, owner) in limbs.iter_mut() {
            if owner.0 != char_e {
                continue;
            }
            match &mut *state {
                LimbState::Attached => {}
                LimbState::Falling { vel, ang_vel } => {
                    all_attached = false;
                    vel.y -= 14.0 * dt;
                    tf.translation += *vel * dt;
                    let spin = Quat::from_euler(
                        EulerRot::XYZ,
                        ang_vel.x * dt,
                        ang_vel.y * dt,
                        ang_vel.z * dt,
                    );
                    tf.rotation = (spin * tf.rotation).normalize();
                    if tf.translation.y <= 0.5 {
                        tf.translation.y = 0.5;
                        *state = LimbState::Lying { timer: 2.0 };
                    }
                }
                LimbState::Lying { timer } => {
                    all_attached = false;
                    *timer -= dt;
                    if *timer <= 0.0 {
                        *state = LimbState::Returning {
                            elapsed: 0.0,
                            start_pos: tf.translation,
                            start_rot: tf.rotation,
                        };
                    }
                }
                LimbState::Returning {
                    elapsed,
                    start_pos,
                    start_rot,
                } => {
                    *elapsed += dt;
                    let dur = 1.0;
                    let t = (*elapsed / dur).min(1.0);
                    let s = t * t * (3.0 - 2.0 * t);
                    let target_matrix = char_gt.compute_matrix() * home.0.compute_matrix();
                    let target = Transform::from_matrix(target_matrix);
                    tf.translation = start_pos.lerp(target.translation, s);
                    tf.rotation = start_rot.slerp(target.rotation, s);
                    if t >= 1.0 {
                        *state = LimbState::Attached;
                        commands.entity(lim_e).set_parent(char_e).insert(home.0);
                    } else {
                        all_attached = false;
                    }
                }
            }
        }
        if all_attached {
            walk.dismembered = false;
            walk.anger = (walk.anger + 0.3).min(1.0);
        }
    }
}

pub fn animate_limbs(
    mut limb_query: Query<(&mut Transform, &Limb, &LimbState, &LimbOwner)>,
    chars: Query<&WalkState, With<Character>>,
) {
    for (mut transform, limb, state, owner) in limb_query.iter_mut() {
        if !matches!(state, LimbState::Attached) {
            continue;
        }
        let Ok(walk) = chars.get(owner.0) else {
            continue;
        };
        let active = !walk.is_staring
            && !walk.held
            && walk.fallen_remaining == 0.0
            && !walk.dismembered
            && !walk.smited
            && walk.dying_timer <= 0.0
            && walk.ascending_remaining <= 0.0;
        let amplitude = if active { 0.6 } else { 0.0 };
        let swing = walk.walk_phase.sin() * amplitude;
        let mut angle = match limb {
            Limb::LeftLeg | Limb::RightArm => swing,
            Limb::RightLeg | Limb::LeftArm => -swing,
        };
        if walk.punch_anim > 0.0 && matches!(limb, Limb::RightArm) {
            let t = (PUNCH_DUR - walk.punch_anim) / PUNCH_DUR;
            let peak = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
            angle = -std::f32::consts::FRAC_PI_2 * peak;
        }
        transform.rotation = Quat::from_rotation_x(angle);
    }
}
