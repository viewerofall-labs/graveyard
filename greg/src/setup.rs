use bevy::prelude::*;

use crate::camera::OrbitCamera;
use crate::character::{spawn_character, CharMeshes, CharacterKind, CHAR_HEIGHT};
use crate::sfx::Sfx;
use crate::torment::spawn_torment_ui;
use crate::ui::{spawn_anger_bar, spawn_hp_bar, spawn_keybind_panel};
use crate::world::{spawn_hazards, spawn_house, spawn_trip_cubes};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Sfx {
        punch: asset_server.load("sfx/punch.wav"),
        smite: asset_server.load("sfx/smite.wav"),
        fall: asset_server.load("sfx/fall.wav"),
        trip: asset_server.load("sfx/trip.wav"),
        door: asset_server.load("sfx/door.wav"),
        dismember: asset_server.load("sfx/dismember.wav"),
        death: asset_server.load("sfx/death.wav"),
        bounce: asset_server.load("sfx/bounce.wav"),
        piano: asset_server.load("sfx/piano.wav"),
        ascend: asset_server.load("sfx/ascend.wav"),
    });

    let radius = 16.0;
    let yaw = 0.0_f32;
    let pitch = 0.5_f32;
    let cam_pos = Vec3::new(
        radius * yaw.sin() * pitch.cos(),
        radius * pitch.sin(),
        radius * yaw.cos() * pitch.cos(),
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(cam_pos).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera { yaw, pitch, radius },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 4.0,
            std::f32::consts::PI / 4.0,
            0.0,
        )),
    ));

    let platform_mesh = meshes.add(Cuboid::new(20.0, 1.0, 20.0));
    let platform_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.18, 0.20),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.spawn((
        Mesh3d(platform_mesh),
        MeshMaterial3d(platform_mat),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    let line_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.35, 0.42),
        perceptual_roughness: 0.8,
        ..default()
    });
    let line_x = meshes.add(Cuboid::new(20.0, 0.02, 0.04));
    let line_z = meshes.add(Cuboid::new(0.04, 0.02, 20.0));
    for i in -5..=5 {
        let p = i as f32 * 2.0;
        commands.spawn((
            Mesh3d(line_x.clone()),
            MeshMaterial3d(line_mat.clone()),
            Transform::from_xyz(0.0, 0.01, p),
        ));
        commands.spawn((
            Mesh3d(line_z.clone()),
            MeshMaterial3d(line_mat.clone()),
            Transform::from_xyz(p, 0.01, 0.0),
        ));
    }

    let cm = CharMeshes {
        torso: meshes.add(Cuboid::new(0.6, 0.8, 0.3)),
        head: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
        arm: meshes.add(Cuboid::new(0.18, 0.7, 0.18)),
        leg: meshes.add(Cuboid::new(0.22, 0.7, 0.22)),
        eye: meshes.add(Cuboid::new(0.06, 0.06, 0.02)),
        horn: meshes.add(Cuboid::new(0.08, 0.25, 0.08)),
        bruise: meshes.add(Cuboid::new(0.7, 1.5, 0.5)),
    };

    let greg_body: Handle<Scene> = asset_server.load("models/body/body_light.gltf#Scene0");
    let fred_body: Handle<Scene> = asset_server.load("models/body/body_dark.gltf#Scene0");
    let hair: Handle<Scene> = asset_server.load("models/hair/Hair_Long.gltf#Scene0");

    let greg_id = spawn_character(
        &mut commands,
        &mut materials,
        &cm,
        CharacterKind::Greg,
        Vec3::new(-3.0, CHAR_HEIGHT, 0.0),
        [0.95, 0.95, 0.95],
        [0.95, 0.20, 0.07],
        Color::srgb(0.05, 0.05, 0.05),
        LinearRgba::rgb(0.0, 0.0, 0.0),
        false,
        Some(greg_body),
        Some(hair.clone()),
    );

    let fred_id = spawn_character(
        &mut commands,
        &mut materials,
        &cm,
        CharacterKind::Fred,
        Vec3::new(3.0, CHAR_HEIGHT, 0.0),
        [0.22, 0.22, 0.26],
        [0.85, 0.05, 0.05],
        Color::srgb(1.0, 0.05, 0.05),
        LinearRgba::rgb(8.0, 0.0, 0.0),
        true,
        Some(fred_body),
        Some(hair),
    );

    spawn_anger_bar(&mut commands, greg_id);
    spawn_anger_bar(&mut commands, fred_id);
    spawn_hp_bar(&mut commands, fred_id);

    spawn_house(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-6.5, 0.0, -6.5),
    );

    spawn_trip_cubes(&mut commands, &mut meshes, &mut materials);
    spawn_hazards(&mut commands, &mut meshes, &mut materials);

    spawn_keybind_panel(&mut commands);
    spawn_torment_ui(&mut commands);
}
