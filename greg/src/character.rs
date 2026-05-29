use bevy::prelude::*;

use crate::limbs::{HomeTransform, Limb, LimbOwner, LimbState};

pub const CHAR_HEIGHT: f32 = 1.1;
pub const PLATFORM_HALF: f32 = 10.0;
pub const LIFT_HEIGHT: f32 = 3.0;
pub const PUNCH_DUR: f32 = 0.35;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharacterKind {
    Greg,
    Fred,
}

#[derive(Component, Clone)]
pub struct Character {
    pub kind: CharacterKind,
    pub body_mat: Handle<StandardMaterial>,
    pub base_color: [f32; 3],
    pub rage_color: [f32; 3],
}

#[derive(Component)]
pub struct WalkState {
    pub target: Vec3,
    pub elapsed_since_stare_check: f32,
    pub staring_elapsed: f32,
    pub is_staring: bool,
    pub walk_phase: f32,
    pub held: bool,
    pub fallen_remaining: f32,
    pub anger: f32,
    pub dismembered: bool,
    pub smited: bool,
    pub smite_timer: f32,
    pub target_timer: f32,
    pub jitter_timer: f32,
    pub jitter_offset: Vec3,
    pub attack_cooldown: f32,
    pub punch_anim: f32,
    pub hp: f32,
    pub dying_timer: f32,
    pub trip_cooldown: f32,
    pub speech_cooldown: f32,
    pub bubble: Option<Entity>,
    pub bounce_remaining: f32,
    pub bounce_from: Vec3,
    pub bounce_to: Vec3,
    pub in_pit: f32,
    pub has_model: bool,
    pub jailed_remaining: f32,
    pub bruise: f32,
    pub ascending_remaining: f32,
}

impl Default for WalkState {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            elapsed_since_stare_check: 0.0,
            staring_elapsed: 0.0,
            is_staring: false,
            walk_phase: 0.0,
            held: false,
            fallen_remaining: 0.0,
            anger: 0.0,
            dismembered: false,
            smited: false,
            smite_timer: 0.0,
            target_timer: 3.0,
            jitter_timer: 0.0,
            jitter_offset: Vec3::ZERO,
            attack_cooldown: 0.0,
            punch_anim: 0.0,
            hp: 1.0,
            dying_timer: 0.0,
            trip_cooldown: 0.0,
            speech_cooldown: 0.0,
            bubble: None,
            bounce_remaining: 0.0,
            bounce_from: Vec3::ZERO,
            bounce_to: Vec3::ZERO,
            in_pit: 0.0,
            has_model: false,
            jailed_remaining: 0.0,
            bruise: 0.0,
            ascending_remaining: 0.0,
        }
    }
}

#[derive(Component)]
pub struct BruiseOverlay {
    pub owner: Entity,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct BodyMesh {
    pub owner: Entity,
}

#[derive(Component)]
pub struct ModelRoot {
    pub owner: Entity,
}

pub struct CharMeshes {
    pub torso: Handle<Mesh>,
    pub head: Handle<Mesh>,
    pub arm: Handle<Mesh>,
    pub leg: Handle<Mesh>,
    pub eye: Handle<Mesh>,
    pub horn: Handle<Mesh>,
    pub bruise: Handle<Mesh>,
}

pub fn spawn_character(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    cm: &CharMeshes,
    kind: CharacterKind,
    pos: Vec3,
    base_color: [f32; 3],
    rage_color: [f32; 3],
    eye_color: Color,
    eye_emissive: LinearRgba,
    has_horns: bool,
    body_scene: Option<Handle<Scene>>,
    hair_scene: Option<Handle<Scene>>,
) -> Entity {
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(base_color[0], base_color[1], base_color[2]),
        perceptual_roughness: 0.7,
        ..default()
    });
    let eye_mat = materials.add(StandardMaterial {
        base_color: eye_color,
        emissive: eye_emissive,
        unlit: matches!(kind, CharacterKind::Fred),
        ..default()
    });
    let bruise_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.04, 0.0, 0.06, 0.0),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        ..default()
    });

    let mut walk = WalkState::default();
    walk.target = random_position();
    walk.has_model = body_scene.is_some();
    if matches!(kind, CharacterKind::Fred) {
        walk.anger = 0.25;
    }

    let parent_id = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            Character {
                kind,
                body_mat: body_mat.clone(),
                base_color,
                rage_color,
            },
            walk,
        ))
        .id();

    commands.entity(parent_id).with_children(|p| {
        p.spawn((
            Mesh3d(cm.bruise.clone()),
            MeshMaterial3d(bruise_mat.clone()),
            Transform::from_xyz(0.0, -0.1, 0.0),
            BruiseOverlay {
                owner: parent_id,
                material: bruise_mat.clone(),
            },
        ));
        p.spawn((
            Mesh3d(cm.torso.clone()),
            MeshMaterial3d(body_mat.clone()),
            BodyMesh { owner: parent_id },
        ));
        p.spawn((
            Mesh3d(cm.head.clone()),
            MeshMaterial3d(body_mat.clone()),
            Transform::from_xyz(0.0, 0.65, 0.0),
            BodyMesh { owner: parent_id },
        ))
        .with_children(|h| {
            h.spawn((
                Mesh3d(cm.eye.clone()),
                MeshMaterial3d(eye_mat.clone()),
                Transform::from_xyz(-0.12, 0.05, -0.26),
                BodyMesh { owner: parent_id },
            ));
            h.spawn((
                Mesh3d(cm.eye.clone()),
                MeshMaterial3d(eye_mat.clone()),
                Transform::from_xyz(0.12, 0.05, -0.26),
                BodyMesh { owner: parent_id },
            ));
            if has_horns {
                h.spawn((
                    Mesh3d(cm.horn.clone()),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(-0.18, 0.30, 0.0)
                        .with_rotation(Quat::from_rotation_z(0.4)),
                    BodyMesh { owner: parent_id },
                ));
                h.spawn((
                    Mesh3d(cm.horn.clone()),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(0.18, 0.30, 0.0)
                        .with_rotation(Quat::from_rotation_z(-0.4)),
                    BodyMesh { owner: parent_id },
                ));
            }
        });
        for (x, lk) in [(-0.39, Limb::LeftArm), (0.39, Limb::RightArm)] {
            let home = Transform::from_xyz(x, 0.3, 0.0);
            p.spawn((
                home,
                Visibility::default(),
                lk,
                HomeTransform(home),
                LimbState::default(),
                LimbOwner(parent_id),
            ))
            .with_children(|a| {
                a.spawn((
                    Mesh3d(cm.arm.clone()),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(0.0, -0.35, 0.0),
                    BodyMesh { owner: parent_id },
                ));
            });
        }
        for (x, lk) in [(-0.15, Limb::LeftLeg), (0.15, Limb::RightLeg)] {
            let home = Transform::from_xyz(x, -0.4, 0.0);
            p.spawn((
                home,
                Visibility::default(),
                lk,
                HomeTransform(home),
                LimbState::default(),
                LimbOwner(parent_id),
            ))
            .with_children(|l| {
                l.spawn((
                    Mesh3d(cm.leg.clone()),
                    MeshMaterial3d(body_mat.clone()),
                    Transform::from_xyz(0.0, -0.35, 0.0),
                    BodyMesh { owner: parent_id },
                ));
            });
        }

        let face_fix = Quat::from_rotation_y(std::f32::consts::PI);
        if let Some(scene) = body_scene {
            p.spawn((
                SceneRoot(scene),
                Transform::from_xyz(0.0, -CHAR_HEIGHT, 0.0).with_rotation(face_fix),
                Visibility::default(),
                ModelRoot { owner: parent_id },
            ));
        }
        if let Some(scene) = hair_scene {
            p.spawn((
                SceneRoot(scene),
                Transform::from_xyz(0.0, -CHAR_HEIGHT, 0.0).with_rotation(face_fix),
                Visibility::default(),
                ModelRoot { owner: parent_id },
            ));
        }
    });

    parent_id
}

pub fn update_model_visibility(
    chars: Query<(Entity, &WalkState), With<Character>>,
    mut bodies: Query<(&BodyMesh, &mut Visibility), Without<ModelRoot>>,
    mut roots: Query<(&ModelRoot, &mut Visibility), Without<BodyMesh>>,
) {
    for (e, w) in chars.iter() {
        // With model: primitives always hidden, model hidden only during dismember
        // Without model: primitives shown unless smited (separate hide), model never shown
        let show_primitive = !w.has_model;
        let show_model = w.has_model && !w.dismembered;
        for (bm, mut vis) in bodies.iter_mut() {
            if bm.owner == e {
                *vis = if show_primitive {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
        for (mr, mut vis) in roots.iter_mut() {
            if mr.owner == e {
                *vis = if show_model {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

pub fn walk_characters(
    mut query: Query<(&mut Transform, &mut WalkState, &Character)>,
    time: Res<Time>,
    cam: Query<&Transform, (With<Camera3d>, Without<Character>)>,
) {
    use crate::world::{BOUNCE_DUR, BOUNCE_PEAK, PIT_DUR};
    let dt = time.delta().as_secs_f32();
    let camera_pos = cam.get_single().ok().map(|t| t.translation);

    let greg_pos = query
        .iter()
        .find_map(|(t, _, ch)| (ch.kind == CharacterKind::Greg).then_some(t.translation));

    for (mut transform, mut walk_state, ch) in query.iter_mut() {
        walk_state.anger = (walk_state.anger - dt * 0.12).max(0.0);

        if walk_state.dismembered
            || walk_state.held
            || walk_state.smited
            || walk_state.dying_timer > 0.0
            || walk_state.jailed_remaining > 0.0
            || walk_state.ascending_remaining > 0.0
        {
            continue;
        }

        if walk_state.bounce_remaining > 0.0 {
            walk_state.bounce_remaining = (walk_state.bounce_remaining - dt).max(0.0);
            let t_norm =
                ((BOUNCE_DUR - walk_state.bounce_remaining) / BOUNCE_DUR).clamp(0.0, 1.0);
            let pos = walk_state.bounce_from.lerp(walk_state.bounce_to, t_norm);
            let h = 4.0 * t_norm * (1.0 - t_norm) * BOUNCE_PEAK;
            transform.translation = Vec3::new(pos.x, CHAR_HEIGHT + h, pos.z);
            transform.rotation = transform.rotation
                * Quat::from_rotation_y(dt * 6.0)
                * Quat::from_rotation_x(dt * 4.0);
            if walk_state.bounce_remaining <= 0.0 {
                transform.translation.y = CHAR_HEIGHT;
                walk_state.target = random_position();
                walk_state.target_timer = 0.4;
                walk_state.fallen_remaining = 0.6;
            }
            continue;
        }

        if walk_state.in_pit > 0.0 {
            walk_state.in_pit = (walk_state.in_pit - dt).max(0.0);
            let p = ((PIT_DUR - walk_state.in_pit) / PIT_DUR).clamp(0.0, 1.0);
            transform.translation.y = CHAR_HEIGHT - p * 4.5;
            if walk_state.in_pit <= 0.0 {
                transform.translation = random_position();
                transform.rotation = Quat::IDENTITY;
                walk_state.target = random_position();
                walk_state.target_timer = 0.4;
                walk_state.anger = 1.0;
                walk_state.fallen_remaining = 1.2;
            }
            continue;
        }

        if walk_state.fallen_remaining > 0.0 {
            walk_state.fallen_remaining = (walk_state.fallen_remaining - dt).max(0.0);
            let total = 2.0;
            let elapsed = total - walk_state.fallen_remaining;
            let fall_dur = 0.3;
            let stand_dur = 0.3;
            let progress = if elapsed < fall_dur {
                elapsed / fall_dur
            } else if elapsed < total - stand_dur {
                1.0
            } else {
                ((total - elapsed) / stand_dur).max(0.0)
            };
            let tilt = progress * std::f32::consts::FRAC_PI_2;
            transform.translation.y = CHAR_HEIGHT - progress * (CHAR_HEIGHT - 0.35);
            let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, -tilt, 0.0);
            continue;
        }

        walk_state.elapsed_since_stare_check += dt;
        if walk_state.elapsed_since_stare_check >= 20.0 {
            walk_state.elapsed_since_stare_check = 0.0;
            if rand::random::<f32>() < 0.1 {
                walk_state.is_staring = true;
                walk_state.staring_elapsed = 0.0;
            }
        }

        if walk_state.is_staring {
            walk_state.staring_elapsed += dt;
            if walk_state.staring_elapsed >= 2.0 {
                walk_state.is_staring = false;
                walk_state.target = random_position();
            }
            if let Some(cam_pos) = camera_pos {
                let y = transform.translation.y;
                let look_target = Vec3::new(cam_pos.x, y, cam_pos.z);
                transform.look_at(look_target, Vec3::Y);
            }
        } else {
            walk_state.target_timer -= dt;
            let pos = transform.translation;
            let to_target = walk_state.target - pos;
            let horizontal = Vec3::new(to_target.x, 0.0, to_target.z);
            let distance = horizontal.length();

            if distance < 0.5 || walk_state.target_timer <= 0.0 {
                walk_state.target = match ch.kind {
                    CharacterKind::Fred => {
                        if let Some(gp) = greg_pos {
                            if rand::random::<f32>() < 0.55 {
                                let off = Vec3::new(
                                    (rand::random::<f32>() - 0.5) * 2.0,
                                    0.0,
                                    (rand::random::<f32>() - 0.5) * 2.0,
                                );
                                Vec3::new(gp.x, CHAR_HEIGHT, gp.z) + off
                            } else {
                                random_position()
                            }
                        } else {
                            random_position()
                        }
                    }
                    CharacterKind::Greg => random_position(),
                };
                walk_state.target_timer = (3.0 / (1.0 + walk_state.anger * 6.0)).max(0.35);
            } else {
                let mut direction = horizontal / distance;

                walk_state.jitter_timer -= dt;
                if walk_state.jitter_timer <= 0.0 {
                    walk_state.jitter_timer = 0.05 + rand::random::<f32>() * 0.15;
                    let s = walk_state.anger;
                    walk_state.jitter_offset = Vec3::new(
                        (rand::random::<f32>() - 0.5) * 1.6 * s,
                        0.0,
                        (rand::random::<f32>() - 0.5) * 1.6 * s,
                    );
                }
                direction = (direction + walk_state.jitter_offset).normalize_or_zero();
                if direction == Vec3::ZERO {
                    direction = horizontal / distance;
                }

                let base_speed = if matches!(ch.kind, CharacterKind::Fred) { 2.4 } else { 2.0 };
                let speed = base_speed + walk_state.anger * 5.5;
                transform.translation += direction * speed * dt;
                transform.translation.y = CHAR_HEIGHT;
                let cx = transform.translation.x.clamp(-PLATFORM_HALF + 0.4, PLATFORM_HALF - 0.4);
                let cz = transform.translation.z.clamp(-PLATFORM_HALF + 0.4, PLATFORM_HALF - 0.4);
                transform.translation.x = cx;
                transform.translation.z = cz;
                let y = transform.translation.y;
                let look_target = transform.translation + direction;
                transform.look_at(Vec3::new(look_target.x, y, look_target.z), Vec3::Y);
                walk_state.walk_phase += dt * (7.0 + walk_state.anger * 12.0);
            }
        }
    }
}

pub fn update_bruise_visual(
    chars: Query<&WalkState, With<Character>>,
    overlays: Query<&BruiseOverlay>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ov in overlays.iter() {
        let Ok(w) = chars.get(ov.owner) else {
            continue;
        };
        let Some(mat) = materials.get_mut(&ov.material) else {
            continue;
        };
        let alpha = w.bruise.clamp(0.0, 1.0) * 0.55;
        mat.base_color = Color::srgba(0.04, 0.0, 0.06, alpha);
    }
}

pub fn update_anger_visual(
    chars: Query<(&WalkState, &Character)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (walk, ch) in chars.iter() {
        let Some(mat) = materials.get_mut(&ch.body_mat) else {
            continue;
        };
        let a = walk.anger.clamp(0.0, 1.0);
        let r = ch.base_color[0] + (ch.rage_color[0] - ch.base_color[0]) * a;
        let g = ch.base_color[1] + (ch.rage_color[1] - ch.base_color[1]) * a;
        let b = ch.base_color[2] + (ch.rage_color[2] - ch.base_color[2]) * a;
        mat.base_color = Color::srgb(r, g, b);
    }
}

pub fn random_position() -> Vec3 {
    let x = (rand::random::<f32>() - 0.5) * 16.0;
    let z = (rand::random::<f32>() - 0.5) * 16.0;
    Vec3::new(x, CHAR_HEIGHT, z)
}
