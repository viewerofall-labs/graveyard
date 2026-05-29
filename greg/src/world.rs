use bevy::prelude::*;

use crate::camera::CameraShake;
use crate::character::{random_position, Character, CharacterKind, WalkState, CHAR_HEIGHT};
use crate::sfx::{play_sfx, Sfx};
use crate::smite::{trigger_smite, SmiteRequests};
use crate::torment::TormentEvent;
use crate::ui::SpeechQueue;

pub const BOUNCE_DUR: f32 = 0.9;
pub const BOUNCE_PEAK: f32 = 5.5;
pub const PIT_DUR: f32 = 0.8;

#[derive(Component)]
pub struct Door {
    pub open: bool,
    pub angle: f32,
    pub pos: Vec3,
}

#[derive(Component)]
pub struct TripCube;

#[derive(Component)]
pub struct LavaPatch;

#[derive(Component)]
pub struct Pit;

#[derive(Component)]
pub struct Trampoline;

#[derive(Component)]
pub struct Piano {
    pub vel_y: f32,
    pub spin: f32,
}

pub fn spawn_house(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
) {
    let house_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.32, 0.22),
        perceptual_roughness: 0.85,
        ..default()
    });
    let roof_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.10, 0.10),
        perceptual_roughness: 0.7,
        ..default()
    });
    let door_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.18, 0.08),
        perceptual_roughness: 0.6,
        ..default()
    });

    let size: f32 = 3.2;
    let h: f32 = 2.4;
    let door_w: f32 = 1.0;
    let door_h: f32 = 1.9;
    let wall_t: f32 = 0.14;
    let half = size / 2.0;
    let front_side_w = (size - door_w) / 2.0;
    let top_h = h - door_h;

    let wall_mesh_back = meshes.add(Cuboid::new(size, h, wall_t));
    let wall_mesh_side = meshes.add(Cuboid::new(wall_t, h, size));
    let wall_mesh_front_side = meshes.add(Cuboid::new(front_side_w, h, wall_t));
    let wall_mesh_lintel = meshes.add(Cuboid::new(door_w, top_h, wall_t));
    let roof_mesh = meshes.add(Cuboid::new(size + 0.25, 0.18, size + 0.25));

    commands.spawn((
        Mesh3d(wall_mesh_back),
        MeshMaterial3d(house_mat.clone()),
        Transform::from_xyz(pos.x, pos.y + h / 2.0, pos.z - half),
    ));
    commands.spawn((
        Mesh3d(wall_mesh_side.clone()),
        MeshMaterial3d(house_mat.clone()),
        Transform::from_xyz(pos.x - half, pos.y + h / 2.0, pos.z),
    ));
    commands.spawn((
        Mesh3d(wall_mesh_side),
        MeshMaterial3d(house_mat.clone()),
        Transform::from_xyz(pos.x + half, pos.y + h / 2.0, pos.z),
    ));
    commands.spawn((
        Mesh3d(wall_mesh_front_side.clone()),
        MeshMaterial3d(house_mat.clone()),
        Transform::from_xyz(
            pos.x - half + front_side_w / 2.0,
            pos.y + h / 2.0,
            pos.z + half,
        ),
    ));
    commands.spawn((
        Mesh3d(wall_mesh_front_side),
        MeshMaterial3d(house_mat.clone()),
        Transform::from_xyz(
            pos.x + half - front_side_w / 2.0,
            pos.y + h / 2.0,
            pos.z + half,
        ),
    ));
    commands.spawn((
        Mesh3d(wall_mesh_lintel),
        MeshMaterial3d(house_mat.clone()),
        Transform::from_xyz(pos.x, pos.y + door_h + top_h / 2.0, pos.z + half),
    ));
    commands.spawn((
        Mesh3d(roof_mesh),
        MeshMaterial3d(roof_mat),
        Transform::from_xyz(pos.x, pos.y + h + 0.09, pos.z),
    ));

    let door_mesh = meshes.add(Cuboid::new(door_w, door_h, 0.06));
    let hinge_pos = Vec3::new(pos.x - door_w / 2.0, pos.y + door_h / 2.0, pos.z + half);
    let door_id = commands
        .spawn((
            Transform::from_translation(hinge_pos),
            Visibility::default(),
            Door {
                open: false,
                angle: 0.0,
                pos: Vec3::new(pos.x, pos.y, pos.z + half),
            },
        ))
        .id();
    commands.entity(door_id).with_children(|d| {
        d.spawn((
            Mesh3d(door_mesh),
            MeshMaterial3d(door_mat),
            Transform::from_xyz(door_w / 2.0, 0.0, 0.0),
        ));
    });
}

pub fn spawn_trip_cubes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let cube_mesh = meshes.add(Cuboid::new(0.45, 0.45, 0.45));
    let cube_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.57, 0.92),
        perceptual_roughness: 0.5,
        ..default()
    });
    let positions = [
        Vec3::new(2.0, 0.225, 4.0),
        Vec3::new(-4.0, 0.225, 3.0),
        Vec3::new(5.0, 0.225, -2.0),
        Vec3::new(0.0, 0.225, -5.0),
        Vec3::new(-2.5, 0.225, -3.0),
        Vec3::new(3.5, 0.225, 1.5),
    ];
    for p in positions {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(cube_mat.clone()),
            Transform::from_translation(p),
            TripCube,
        ));
    }
}

pub fn spawn_hazards(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let lava_mesh = meshes.add(Cuboid::new(1.6, 0.04, 1.6));
    let lava_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.4, 0.05),
        emissive: LinearRgba::rgb(5.0, 1.4, 0.0),
        unlit: true,
        ..default()
    });
    for p in [Vec3::new(6.0, 0.03, 3.0), Vec3::new(-5.5, 0.03, -2.5)] {
        commands.spawn((
            Mesh3d(lava_mesh.clone()),
            MeshMaterial3d(lava_mat.clone()),
            Transform::from_translation(p),
            LavaPatch,
        ));
    }

    let pit_mesh = meshes.add(Cuboid::new(1.8, 0.06, 1.8));
    let pit_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.02, 0.0, 0.04),
        perceptual_roughness: 1.0,
        ..default()
    });
    let pit_rim = meshes.add(Cuboid::new(2.1, 0.02, 2.1));
    let pit_rim_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.0, 0.10),
        emissive: LinearRgba::rgb(0.3, 0.0, 0.4),
        unlit: true,
        ..default()
    });
    let pit_pos = Vec3::new(-2.0, 0.0, 6.0);
    commands.spawn((
        Mesh3d(pit_rim),
        MeshMaterial3d(pit_rim_mat),
        Transform::from_xyz(pit_pos.x, 0.04, pit_pos.z),
    ));
    commands.spawn((
        Mesh3d(pit_mesh),
        MeshMaterial3d(pit_mat),
        Transform::from_xyz(pit_pos.x, 0.03, pit_pos.z),
        Pit,
    ));

    let tramp_mesh = meshes.add(Cuboid::new(1.4, 0.4, 1.4));
    let tramp_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.9, 0.78),
        emissive: LinearRgba::rgb(0.0, 1.6, 1.2),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(tramp_mesh),
        MeshMaterial3d(tramp_mat),
        Transform::from_xyz(4.5, 0.22, -5.0),
        Trampoline,
    ));
}

pub fn trip_cubes_system(
    mut chars: Query<(Entity, &Transform, &mut WalkState, &Character)>,
    cubes: Query<&Transform, (With<TripCube>, Without<Character>)>,
    mut shake: ResMut<CameraShake>,
    mut commands: Commands,
    mut speech: ResMut<SpeechQueue>,
    mut torment_ev: EventWriter<TormentEvent>,
    sfx: Res<Sfx>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (e, t, mut walk, ch) in chars.iter_mut() {
        if walk.trip_cooldown > 0.0 {
            walk.trip_cooldown -= dt;
        }
        if walk.dismembered
            || walk.smited
            || walk.held
            || walk.dying_timer > 0.0
            || walk.fallen_remaining > 0.0
            || walk.bounce_remaining > 0.0
            || walk.in_pit > 0.0
            || walk.trip_cooldown > 0.0
            || walk.jailed_remaining > 0.0
            || walk.ascending_remaining > 0.0
        {
            continue;
        }
        for cube_t in cubes.iter() {
            let d = Vec3::new(
                t.translation.x - cube_t.translation.x,
                0.0,
                t.translation.z - cube_t.translation.z,
            )
            .length();
            if d < 0.5 {
                walk.fallen_remaining = 1.5;
                walk.anger = (walk.anger + 0.3).min(1.0);
                walk.bruise = (walk.bruise + 0.04).min(1.0);
                walk.trip_cooldown = 3.5;
                shake.intensity = shake.intensity.max(0.18);
                if matches!(ch.kind, CharacterKind::Greg) {
                    torment_ev.send(TormentEvent::Trip);
                }
                play_sfx(&mut commands, &sfx.trip);
                let color = match ch.kind {
                    CharacterKind::Greg => [0.95, 0.95, 0.95],
                    CharacterKind::Fred => [1.0, 0.3, 0.3],
                };
                speech.0.push((e, "oof".into(), color));
                break;
            }
        }
    }
}

pub fn update_door(
    chars: Query<(&Transform, &Character)>,
    mut doors: Query<(&mut Transform, &mut Door), Without<Character>>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (mut t, mut door) in doors.iter_mut() {
        let mut near = false;
        for (ct, _) in chars.iter() {
            let d = Vec3::new(
                ct.translation.x - door.pos.x,
                0.0,
                ct.translation.z - door.pos.z,
            )
            .length();
            if d < 2.0 {
                near = true;
                break;
            }
        }
        if near != door.open {
            play_sfx(&mut commands, &sfx.door);
        }
        door.open = near;
        let target = if door.open {
            -std::f32::consts::FRAC_PI_2 * 0.95
        } else {
            0.0
        };
        let speed = 4.0;
        let delta = (target - door.angle).clamp(-speed * dt, speed * dt);
        door.angle += delta;
        t.rotation = Quat::from_rotation_y(door.angle);
    }
}

pub fn piano_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !keys.just_pressed(KeyCode::KeyP) {
        return;
    }
    spawn_piano(&mut commands, &mut meshes, &mut materials);
}

fn spawn_piano(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.04, 0.02, 0.06),
        perceptual_roughness: 0.35,
        metallic: 0.15,
        ..default()
    });
    let key_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.92),
        emissive: LinearRgba::rgb(0.4, 0.4, 0.55),
        unlit: true,
        ..default()
    });
    let trim_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.57, 0.92),
        emissive: LinearRgba::rgb(0.6, 0.3, 0.85),
        unlit: true,
        ..default()
    });

    let body_mesh = meshes.add(Cuboid::new(1.2, 0.75, 0.7));
    let lid_mesh = meshes.add(Cuboid::new(1.3, 0.10, 0.78));
    let keys_mesh = meshes.add(Cuboid::new(1.15, 0.05, 0.22));
    let trim_mesh = meshes.add(Cuboid::new(1.22, 0.03, 0.72));

    let x = (rand::random::<f32>() - 0.5) * 14.0;
    let z = (rand::random::<f32>() - 0.5) * 14.0;
    let spin = (rand::random::<f32>() - 0.5) * 2.5;

    commands
        .spawn((
            Transform::from_xyz(x, 16.0, z),
            Visibility::default(),
            Piano { vel_y: 0.0, spin },
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(body_mesh),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            p.spawn((
                Mesh3d(lid_mesh),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 0.42, 0.0),
            ));
            p.spawn((
                Mesh3d(keys_mesh),
                MeshMaterial3d(key_mat),
                Transform::from_xyz(0.0, -0.06, 0.27),
            ));
            p.spawn((
                Mesh3d(trim_mesh),
                MeshMaterial3d(trim_mat),
                Transform::from_xyz(0.0, 0.39, 0.04),
            ));
        });
}

pub fn update_piano(
    mut commands: Commands,
    mut pianos: Query<(Entity, &mut Transform, &mut Piano), Without<Character>>,
    mut chars: Query<(Entity, &mut Transform, &mut WalkState, &Character), With<Character>>,
    mut shake: ResMut<CameraShake>,
    mut speech: ResMut<SpeechQueue>,
    mut torment_ev: EventWriter<TormentEvent>,
    sfx: Res<Sfx>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (pe, mut pt, mut piano) in pianos.iter_mut() {
        piano.vel_y -= 22.0 * dt;
        pt.translation.y += piano.vel_y * dt;
        pt.rotation =
            pt.rotation * Quat::from_rotation_z(piano.spin * dt) * Quat::from_rotation_x(dt * 0.6);

        let mut hit_target: Option<Entity> = None;
        for (ce, ct, w, _) in chars.iter() {
            if w.ascending_remaining > 0.0
                || w.smited
                || w.dying_timer > 0.0
                || w.in_pit > 0.0
            {
                continue;
            }
            let dy = pt.translation.y - ct.translation.y;
            let horiz = Vec3::new(
                pt.translation.x - ct.translation.x,
                0.0,
                pt.translation.z - ct.translation.z,
            )
            .length();
            if dy.abs() < 1.0 && horiz < 0.95 {
                hit_target = Some(ce);
                break;
            }
        }

        if let Some(ce) = hit_target {
            if let Ok((_, mut ct, mut w, ch)) = chars.get_mut(ce) {
                w.ascending_remaining = 1.8;
                w.held = false;
                w.fallen_remaining = 0.0;
                w.bounce_remaining = 0.0;
                w.is_staring = false;
                w.in_pit = 0.0;
                w.jailed_remaining = 0.0;
                w.bruise = (w.bruise + 0.30).min(1.0);
                ct.rotation = Quat::IDENTITY;
                if matches!(ch.kind, CharacterKind::Greg) {
                    torment_ev.send(TormentEvent::Piano);
                }
                let (line, color) = match ch.kind {
                    CharacterKind::Greg => ("a piano???", [0.95, 0.95, 0.95]),
                    CharacterKind::Fred => ("WHAT THE", [1.0, 0.3, 0.3]),
                };
                speech.0.push((ce, line.into(), color));
            }
            shake.intensity = shake.intensity.max(1.0);
            play_sfx(&mut commands, &sfx.piano);
            play_sfx(&mut commands, &sfx.ascend);
            commands.entity(pe).despawn_recursive();
            continue;
        }

        if pt.translation.y <= 0.4 {
            shake.intensity = shake.intensity.max(0.65);
            play_sfx(&mut commands, &sfx.piano);
            commands.entity(pe).despawn_recursive();
        }
    }
}

pub fn update_ascending(
    mut chars: Query<(&mut Transform, &mut WalkState, &Character)>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (mut t, mut w, _ch) in chars.iter_mut() {
        if w.ascending_remaining <= 0.0 {
            continue;
        }
        let was_remaining = w.ascending_remaining;
        w.ascending_remaining = (w.ascending_remaining - dt).max(0.0);

        // rise + shrink + spin
        t.translation.y += 7.5 * dt;
        t.rotation = t.rotation * Quat::from_rotation_y(dt * 3.0);
        let progress = 1.0 - (w.ascending_remaining / 1.8).max(0.0);
        let s = (1.0 - progress * 0.85).max(0.15);
        t.scale = Vec3::splat(s);

        if w.ascending_remaining <= 0.0 && was_remaining > 0.0 {
            t.translation = random_position();
            t.rotation = Quat::IDENTITY;
            t.scale = Vec3::ONE;
            w.anger = 1.0;
            w.target = random_position();
            w.target_timer = 0.4;
            w.fallen_remaining = 0.6;
            play_sfx(&mut commands, &sfx.fall);
        }
    }
}

pub fn hazards_system(
    mut chars: Query<(Entity, &mut Transform, &mut Visibility, &mut WalkState, &Character)>,
    lavas: Query<&Transform, (With<LavaPatch>, Without<Character>)>,
    pits: Query<&Transform, (With<Pit>, Without<Character>, Without<LavaPatch>)>,
    tramps: Query<
        &Transform,
        (
            With<Trampoline>,
            Without<Character>,
            Without<LavaPatch>,
            Without<Pit>,
        ),
    >,
    mut shake: ResMut<CameraShake>,
    mut requests: ResMut<SmiteRequests>,
    mut speech: ResMut<SpeechQueue>,
    mut torment_ev: EventWriter<TormentEvent>,
    mut commands: Commands,
    sfx: Res<Sfx>,
) {
    let lava_positions: Vec<Vec3> = lavas.iter().map(|t| t.translation).collect();
    let pit_positions: Vec<Vec3> = pits.iter().map(|t| t.translation).collect();
    let tramp_positions: Vec<Vec3> = tramps.iter().map(|t| t.translation).collect();

    for (e, mut t, mut vis, mut walk, ch) in chars.iter_mut() {
        if walk.dismembered
            || walk.smited
            || walk.dying_timer > 0.0
            || walk.held
            || walk.bounce_remaining > 0.0
            || walk.in_pit > 0.0
            || walk.fallen_remaining > 0.0
            || walk.jailed_remaining > 0.0
            || walk.ascending_remaining > 0.0
        {
            continue;
        }
        let p = t.translation;

        let mut on_lava = false;
        for lp in &lava_positions {
            if (p.x - lp.x).abs() < 0.8 && (p.z - lp.z).abs() < 0.8 {
                on_lava = true;
                break;
            }
        }
        if on_lava {
            trigger_smite(e, p, &mut walk, &mut vis, &mut shake, &mut requests);
            if matches!(ch.kind, CharacterKind::Greg) {
                torment_ev.send(TormentEvent::Lava);
            }
            let color = match ch.kind {
                CharacterKind::Greg => [0.95, 0.95, 0.95],
                CharacterKind::Fred => [1.0, 0.3, 0.3],
            };
            speech.0.push((e, "HOT".into(), color));
            continue;
        }

        let mut on_pit = false;
        for pp in &pit_positions {
            if (p.x - pp.x).abs() < 0.9 && (p.z - pp.z).abs() < 0.9 {
                on_pit = true;
                break;
            }
        }
        if on_pit {
            walk.in_pit = PIT_DUR;
            walk.held = false;
            walk.is_staring = false;
            shake.intensity = shake.intensity.max(0.4);
            if matches!(ch.kind, CharacterKind::Greg) {
                torment_ev.send(TormentEvent::Pit);
            }
            play_sfx(&mut commands, &sfx.fall);
            let color = match ch.kind {
                CharacterKind::Greg => [0.95, 0.95, 0.95],
                CharacterKind::Fred => [1.0, 0.3, 0.3],
            };
            speech.0.push((e, "WAAAGH".into(), color));
            continue;
        }

        let mut on_tramp = false;
        for tp in &tramp_positions {
            if (p.x - tp.x).abs() < 0.7 && (p.z - tp.z).abs() < 0.7 {
                on_tramp = true;
                break;
            }
        }
        if on_tramp {
            walk.bounce_remaining = BOUNCE_DUR;
            walk.bounce_from = p;
            walk.bounce_to = random_position();
            walk.is_staring = false;
            walk.anger = (walk.anger + 0.4).min(1.0);
            walk.bruise = (walk.bruise + 0.03).min(1.0);
            shake.intensity = shake.intensity.max(0.25);
            if matches!(ch.kind, CharacterKind::Greg) {
                torment_ev.send(TormentEvent::Bounce);
            }
            play_sfx(&mut commands, &sfx.bounce);
            t.translation.y = CHAR_HEIGHT + 0.05;
            let color = match ch.kind {
                CharacterKind::Greg => [0.95, 0.95, 0.95],
                CharacterKind::Fred => [1.0, 0.3, 0.3],
            };
            speech.0.push((e, "WHEE".into(), color));
        }
    }
}
