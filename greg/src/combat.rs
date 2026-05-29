use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::{ray_plane_y, ray_sphere, CameraShake};
use crate::character::{
    random_position, Character, CharacterKind, WalkState, CHAR_HEIGHT, LIFT_HEIGHT,
    PLATFORM_HALF, PUNCH_DUR,
};
use crate::sfx::{play_sfx, Sfx};
use crate::torment::{HitStop, Torment, TormentEvent};
use crate::ui::{pick, SpeechQueue, FRED_HIT_LINES, GREG_COUNTER_LINES, GREG_HURT_LINES};

#[derive(Resource, Default, Clone, Copy)]
pub enum MouseAction {
    #[default]
    Idle,
    Pressed {
        target: Entity,
        start_screen: Vec2,
    },
    Dragging {
        target: Entity,
    },
}

#[derive(Resource, Default)]
pub struct SlowMo {
    pub remaining: f32,
}

pub fn fred_attacks_greg(
    mut chars: Query<(Entity, &Transform, &mut WalkState, &Character)>,
    mut shake: ResMut<CameraShake>,
    mut commands: Commands,
    mut speech: ResMut<SpeechQueue>,
    mut slowmo: ResMut<SlowMo>,
    mut torment_ev: EventWriter<TormentEvent>,
    sfx: Res<Sfx>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();

    let mut greg: Option<(Entity, Vec3, f32, bool, bool, f32)> = None;
    let mut fred: Option<(Entity, Vec3, f32, bool, f32)> = None;
    for (e, t, w, ch) in chars.iter() {
        match ch.kind {
            CharacterKind::Greg => {
                greg = Some((
                    e,
                    t.translation,
                    w.fallen_remaining,
                    w.dismembered || w.jailed_remaining > 0.0 || w.ascending_remaining > 0.0,
                    w.smited,
                    w.dying_timer,
                ));
            }
            CharacterKind::Fred => {
                let ok = !w.dismembered
                    && !w.smited
                    && !w.held
                    && w.fallen_remaining == 0.0
                    && w.dying_timer <= 0.0
                    && w.jailed_remaining <= 0.0
                    && w.ascending_remaining <= 0.0;
                fred = Some((e, t.translation, w.attack_cooldown, ok, w.dying_timer));
            }
        }
    }

    for (_, _, mut w, _) in chars.iter_mut() {
        if w.attack_cooldown > 0.0 {
            w.attack_cooldown -= dt;
        }
        if w.punch_anim > 0.0 {
            w.punch_anim -= dt;
        }
    }

    if let (
        Some((greg_e, greg_p, greg_fall, greg_dis, greg_smt, greg_dying)),
        Some((fred_e, fred_p, fred_cd, fred_ok, fred_dying)),
    ) = (greg, fred)
    {
        let in_range = greg_p.distance(fred_p) < 1.5;
        let greg_attackable =
            greg_fall <= 0.0 && !greg_dis && !greg_smt && greg_dying <= 0.0;
        if fred_ok && in_range && fred_cd <= 0.0 && greg_attackable {
            if let Ok((_, _, mut fw, _)) = chars.get_mut(fred_e) {
                fw.attack_cooldown = 1.6;
                fw.punch_anim = PUNCH_DUR;
            }
            if let Ok((_, _, mut gw, _)) = chars.get_mut(greg_e) {
                gw.fallen_remaining = 1.6;
                gw.anger = (gw.anger + 0.5).min(1.0);
                gw.bruise = (gw.bruise + 0.06).min(1.0);
            }
            shake.intensity = shake.intensity.max(0.3);
            torment_ev.send(TormentEvent::Punch);
            play_sfx(&mut commands, &sfx.punch);
            speech
                .0
                .push((fred_e, pick(FRED_HIT_LINES).into(), [1.0, 0.6, 0.6]));
            speech
                .0
                .push((greg_e, pick(GREG_HURT_LINES).into(), [0.95, 0.95, 0.95]));

            if rand::random::<f32>() < 1.0 / 3.0 && fred_dying <= 0.0 {
                if let Ok((_, _, mut gw, _)) = chars.get_mut(greg_e) {
                    gw.punch_anim = PUNCH_DUR;
                    gw.anger = (gw.anger + 0.2).min(1.0);
                }
                let mut fred_killed = false;
                if let Ok((_, _, mut fw, _)) = chars.get_mut(fred_e) {
                    fw.hp = (fw.hp - 0.20).max(0.0);
                    fw.fallen_remaining = 1.6;
                    fw.anger = (fw.anger + 0.4).min(1.0);
                    fw.bruise = (fw.bruise + 0.10).min(1.0);
                    if fw.hp <= 0.0 {
                        fw.dying_timer = 2.0;
                        fw.fallen_remaining = 0.0;
                        fred_killed = true;
                    }
                }
                shake.intensity = shake.intensity.max(0.45);
                slowmo.remaining = 0.5;
                play_sfx(&mut commands, &sfx.punch);
                speech
                    .0
                    .push((greg_e, pick(GREG_COUNTER_LINES).into(), [0.4, 1.0, 0.6]));
                if fred_killed {
                    play_sfx(&mut commands, &sfx.death);
                    speech
                        .0
                        .push((fred_e, "...the abyss...".into(), [1.0, 0.3, 0.3]));
                }
            }
        }
    }
}

pub fn interact_with_characters(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut chars: Query<(Entity, &mut Transform, &mut WalkState, &Character)>,
    mut action: ResMut<MouseAction>,
    mut shake: ResMut<CameraShake>,
    mut commands: Commands,
    mut speech: ResMut<SpeechQueue>,
    mut torment_ev: EventWriter<TormentEvent>,
    sfx: Res<Sfx>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_xform)) = camera_query.get_single() else {
        return;
    };

    let cursor_pos = window.cursor_position();
    let ray = cursor_pos.and_then(|c| camera.viewport_to_world(cam_xform, c).ok());

    if mouse.just_pressed(MouseButton::Left) {
        if let (Some(cursor), Some(r)) = (cursor_pos, ray) {
            let mut best: Option<(Entity, f32)> = None;
            for (e, t, w, _) in chars.iter() {
                if w.dismembered
                    || w.smited
                    || w.fallen_remaining > 0.0
                    || w.dying_timer > 0.0
                    || w.jailed_remaining > 0.0
                    || w.ascending_remaining > 0.0
                {
                    continue;
                }
                // 3-sphere capsule check: feet/center/head
                let pos = t.translation;
                let candidates = [
                    (pos + Vec3::new(0.0, -0.6, 0.0), 0.7),
                    (pos, 0.9),
                    (pos + Vec3::new(0.0, 0.5, 0.0), 0.7),
                ];
                let mut nearest: Option<f32> = None;
                for (c, rad) in candidates {
                    if let Some(d) = ray_sphere(r, c, rad) {
                        nearest = Some(nearest.map_or(d, |n| n.min(d)));
                    }
                }
                if let Some(dist) = nearest {
                    if best.map_or(true, |(_, d)| dist < d) {
                        best = Some((e, dist));
                    }
                }
            }
            if let Some((target, _)) = best {
                *action = MouseAction::Pressed {
                    target,
                    start_screen: cursor,
                };
            }
        }
    }

    if mouse.pressed(MouseButton::Left) {
        match *action {
            MouseAction::Pressed { target, start_screen } => {
                if let Some(c) = cursor_pos {
                    if (c - start_screen).length() > 8.0 {
                        if let Ok((_, _, mut w, _)) = chars.get_mut(target) {
                            w.held = true;
                            w.is_staring = false;
                        }
                        *action = MouseAction::Dragging { target };
                    }
                }
            }
            MouseAction::Dragging { target } => {
                if let Some(r) = ray {
                    if let Some(hit) = ray_plane_y(r, LIFT_HEIGHT) {
                        if let Ok((_, mut t, _, _)) = chars.get_mut(target) {
                            t.translation = Vec3::new(hit.x, LIFT_HEIGHT, hit.z);
                            let cam_pos = cam_xform.translation();
                            let look_target = Vec3::new(cam_pos.x, LIFT_HEIGHT, cam_pos.z);
                            t.look_at(look_target, Vec3::Y);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if mouse.just_released(MouseButton::Left) {
        match *action {
            MouseAction::Pressed { target, .. } => {
                let mut greg_hit = false;
                if let Ok((_, _, mut w, ch)) = chars.get_mut(target) {
                    if !w.dismembered
                        && !w.smited
                        && w.jailed_remaining <= 0.0
                        && w.ascending_remaining <= 0.0
                    {
                        w.fallen_remaining = 2.0;
                        w.anger = (w.anger + 0.55).min(1.0);
                        w.bruise = (w.bruise + 0.07).min(1.0);
                        shake.intensity = shake.intensity.max(0.25);
                        play_sfx(&mut commands, &sfx.punch);
                        greg_hit = matches!(ch.kind, CharacterKind::Greg);
                    }
                }
                if greg_hit {
                    torment_ev.send(TormentEvent::Punch);
                }
            }
            MouseAction::Dragging { target } => {
                let (drop_pos, drop_kind, off_platform) =
                    if let Ok((_, t, _, ch)) = chars.get(target) {
                        let p = t.translation;
                        let off = p.x.abs() > PLATFORM_HALF || p.z.abs() > PLATFORM_HALF;
                        (p, ch.kind, off)
                    } else {
                        *action = MouseAction::Idle;
                        return;
                    };

                let combo_target = if matches!(drop_kind, CharacterKind::Fred) {
                    chars.iter().find_map(|(e, t, _, ch)| {
                        if matches!(ch.kind, CharacterKind::Greg)
                            && t.translation.distance(drop_pos) < 1.8
                        {
                            Some(e)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                if let Ok((_, mut t, mut w, _)) = chars.get_mut(target) {
                    w.held = false;
                    if off_platform {
                        t.translation = random_position();
                        w.target = random_position();
                        w.anger = 1.0;
                        w.fallen_remaining = 1.5;
                        shake.intensity = shake.intensity.max(0.4);
                        play_sfx(&mut commands, &sfx.fall);
                    } else {
                        t.translation.y = CHAR_HEIGHT;
                        w.target = random_position();
                        w.anger = (w.anger + 0.3).min(1.0);
                        shake.intensity = shake.intensity.max(0.15);
                        play_sfx(&mut commands, &sfx.fall);
                    }
                }

                if let Some(greg_e) = combo_target {
                    if let Ok((_, _, mut gw, _)) = chars.get_mut(greg_e) {
                        gw.fallen_remaining = 2.5;
                        gw.anger = 1.0;
                        gw.bruise = (gw.bruise + 0.18).min(1.0);
                    }
                    shake.intensity = shake.intensity.max(0.9);
                    torment_ev.send(TormentEvent::Yeet);
                    play_sfx(&mut commands, &sfx.punch);
                    speech
                        .0
                        .push((greg_e, "FRED?!".into(), [0.95, 0.95, 0.95]));
                    speech
                        .0
                        .push((target, "COLLIDE".into(), [1.0, 0.3, 0.3]));
                }
            }
            _ => {}
        }
        *action = MouseAction::Idle;
    }
}

pub fn update_death(
    mut chars: Query<(&mut Transform, &mut WalkState, &Character)>,
    mut shake: ResMut<CameraShake>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (mut t, mut walk, _ch) in chars.iter_mut() {
        if walk.dying_timer <= 0.0 {
            continue;
        }
        walk.dying_timer -= dt;
        if walk.dying_timer > 0.0 {
            let progress =
                ((2.0 - walk.dying_timer) / 0.4).clamp(0.0, 1.0);
            let tilt = progress * std::f32::consts::FRAC_PI_2;
            t.translation.y = CHAR_HEIGHT - progress * (CHAR_HEIGHT - 0.35);
            let yaw = t.rotation.to_euler(EulerRot::YXZ).0;
            t.rotation = Quat::from_euler(EulerRot::YXZ, yaw, -tilt, 0.0);
        } else {
            walk.dying_timer = 0.0;
            walk.hp = 1.0;
            walk.anger = 0.6;
            walk.bruise = 0.0;
            walk.fallen_remaining = 0.0;
            walk.held = false;
            walk.target = random_position();
            walk.target_timer = 0.4;
            t.translation = random_position();
            t.rotation = Quat::IDENTITY;
            shake.intensity = shake.intensity.max(0.6);
            play_sfx(&mut commands, &sfx.fall);
        }
    }
}

pub fn update_timescale(
    mut sm: ResMut<SlowMo>,
    mut hs: ResMut<HitStop>,
    mut virt: ResMut<Time<Virtual>>,
    real: Res<Time<Real>>,
    torment: Res<Torment>,
) {
    let dt = real.delta().as_secs_f32();

    if torment.game_over {
        if (virt.relative_speed() - 0.0).abs() > 1e-3 {
            virt.set_relative_speed(0.0);
        }
        return;
    }

    if hs.remaining > 0.0 {
        hs.remaining = (hs.remaining - dt).max(0.0);
        virt.set_relative_speed(0.0);
        return;
    }

    if sm.remaining > 0.0 {
        sm.remaining = (sm.remaining - dt).max(0.0);
        virt.set_relative_speed(0.25);
    } else if (virt.relative_speed() - 1.0).abs() > 1e-3 {
        virt.set_relative_speed(1.0);
    }
}
