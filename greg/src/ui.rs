use bevy::prelude::*;

use crate::character::{Character, CharacterKind, WalkState};

#[derive(Component)]
pub struct AngerBar {
    pub owner: Entity,
    pub fill: Entity,
}

#[derive(Component)]
pub struct AngerBarFill;

#[derive(Component)]
pub struct HpBar {
    pub owner: Entity,
    pub fill: Entity,
}

#[derive(Component)]
pub struct HpBarFill;

#[derive(Resource, Default)]
pub struct SpeechQueue(pub Vec<(Entity, String, [f32; 3])>);

#[derive(Component)]
pub struct SpeechBubble {
    pub owner: Entity,
    pub lifetime: f32,
}

pub const GREG_IDLE: &[&str] = &[
    "ow.",
    "why am I here",
    "leave me alone",
    "is anyone seeing this",
    "i hate this",
    "MOM",
    "please stop",
    "this is fine",
    "what did i do",
    "i just want to be",
];

pub const FRED_IDLE: &[&str] = &[
    "HEHEHE",
    "GREG. die.",
    "evil energy",
    "muahaha",
    "i live to torment",
    ":)",
    "fall down boy",
    "BEHOLD",
    "you cannot escape",
    "skull emoji",
];

pub const GREG_HURT_LINES: &[&str] = &["OW", "STOP", "WHY", "no.", "ow."];
pub const GREG_COUNTER_LINES: &[&str] = &["GET REKT", "TAKE THAT", "EAT FIST", "for once"];
pub const FRED_HIT_LINES: &[&str] = &["EAT IT", "HA", "WEAK", "GAMING", "BONK"];

pub fn pick(lines: &'static [&'static str]) -> &'static str {
    lines[rand::random::<usize>() % lines.len()]
}

pub fn spawn_anger_bar(commands: &mut Commands, owner: Entity) {
    let fill = commands
        .spawn((
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.95, 0.2, 0.07)),
            AngerBarFill,
        ))
        .id();
    let parent = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-9999.0),
                top: Val::Px(-9999.0),
                width: Val::Px(70.0),
                height: Val::Px(6.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.02, 0.05, 0.7)),
            BorderColor(Color::srgba(0.78, 0.57, 0.92, 0.7)),
            AngerBar { owner, fill },
        ))
        .id();
    commands.entity(parent).add_child(fill);
}

pub fn spawn_hp_bar(commands: &mut Commands, owner: Entity) {
    let fill = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.0, 0.9, 0.5)),
            HpBarFill,
        ))
        .id();
    let parent = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-9999.0),
                top: Val::Px(-9999.0),
                width: Val::Px(70.0),
                height: Val::Px(6.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.7)),
            BorderColor(Color::srgba(0.0, 0.9, 0.78, 0.7)),
            HpBar { owner, fill },
        ))
        .id();
    commands.entity(parent).add_child(fill);
}

pub fn spawn_keybind_panel(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                bottom: Val::Px(14.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(3.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.0, 0.06, 0.82)),
            BorderColor(Color::srgba(0.78, 0.57, 0.92, 0.85)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("KEYBINDS"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.9, 0.78)),
            ));
            for line in [
                "LMB click       punch",
                "LMB drag        grab / yeet",
                "LMB drag Fred   yeet onto Greg = combo",
                "RMB drag        orbit cam",
                "Scroll          zoom",
                "J  (hold)       jail Greg",
                "K  (hold)       jail Fred",
                "D               smite Greg",
                "C               smite Fred",
                "P               drop a piano",
                "purple cubes    trip",
                "orange lava     instant smite",
                "black pit       fall in & respawn",
                "cyan pad        trampoline",
            ] {
                p.spawn((
                    Text::new(line),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.85, 0.92)),
                ));
            }
        });
}

pub fn update_bars(
    chars: Query<(&Transform, &WalkState, &Character)>,
    camera: Query<(&Camera, &GlobalTransform), Without<Character>>,
    mut bars: Query<(&mut Node, &AngerBar), (Without<AngerBarFill>, Without<HpBar>, Without<HpBarFill>)>,
    mut hp_bars: Query<(&mut Node, &HpBar), (Without<AngerBar>, Without<AngerBarFill>, Without<HpBarFill>)>,
    mut fills: Query<&mut Node, (With<AngerBarFill>, Without<HpBarFill>, Without<AngerBar>, Without<HpBar>)>,
    mut hp_fills: Query<&mut Node, (With<HpBarFill>, Without<AngerBarFill>, Without<AngerBar>, Without<HpBar>)>,
) {
    let Ok((cam, cam_xform)) = camera.get_single() else {
        return;
    };
    for (mut node, bar) in bars.iter_mut() {
        let Ok((t, walk, _)) = chars.get(bar.owner) else {
            continue;
        };
        let head_world = t.translation + Vec3::new(0.0, 1.1, 0.0);
        if let Ok(screen) = cam.world_to_viewport(cam_xform, head_world) {
            node.left = Val::Px(screen.x - 35.0);
            node.top = Val::Px(screen.y - 22.0);
            if let Ok(mut fill) = fills.get_mut(bar.fill) {
                fill.width = Val::Percent(walk.anger.clamp(0.0, 1.0) * 100.0);
            }
        } else {
            node.left = Val::Px(-9999.0);
            node.top = Val::Px(-9999.0);
        }
    }
    for (mut node, bar) in hp_bars.iter_mut() {
        let Ok((t, walk, _)) = chars.get(bar.owner) else {
            continue;
        };
        let head_world = t.translation + Vec3::new(0.0, 1.25, 0.0);
        if let Ok(screen) = cam.world_to_viewport(cam_xform, head_world) {
            node.left = Val::Px(screen.x - 35.0);
            node.top = Val::Px(screen.y - 12.0);
            if let Ok(mut fill) = hp_fills.get_mut(bar.fill) {
                fill.width = Val::Percent(walk.hp.clamp(0.0, 1.0) * 100.0);
            }
        } else {
            node.left = Val::Px(-9999.0);
            node.top = Val::Px(-9999.0);
        }
    }
}

pub fn idle_speech(
    mut queue: ResMut<SpeechQueue>,
    mut chars: Query<(Entity, &mut WalkState, &Character)>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    for (e, mut w, ch) in chars.iter_mut() {
        if w.speech_cooldown > 0.0 {
            w.speech_cooldown = (w.speech_cooldown - dt).max(0.0);
            continue;
        }
        if w.dismembered
            || w.smited
            || w.dying_timer > 0.0
            || w.held
            || w.jailed_remaining > 0.0
            || w.ascending_remaining > 0.0
        {
            continue;
        }
        if rand::random::<f32>() < dt * 0.18 {
            let (line, color) = match ch.kind {
                CharacterKind::Greg => (pick(GREG_IDLE), [0.95, 0.95, 0.95]),
                CharacterKind::Fred => (pick(FRED_IDLE), [1.0, 0.5, 0.5]),
            };
            queue.0.push((e, line.into(), color));
            w.speech_cooldown = 4.0;
        }
    }
}

pub fn process_speech_queue(
    mut commands: Commands,
    mut queue: ResMut<SpeechQueue>,
    mut chars: Query<&mut WalkState, With<Character>>,
    bubbles: Query<Entity, With<SpeechBubble>>,
) {
    for (owner, text, color) in queue.0.drain(..) {
        let Ok(mut walk) = chars.get_mut(owner) else {
            continue;
        };
        if let Some(prev) = walk.bubble.take() {
            if bubbles.get(prev).is_ok() {
                commands.entity(prev).despawn_recursive();
            }
        }
        let id = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-9999.0),
                    top: Val::Px(-9999.0),
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.0, 0.06, 0.88)),
                BorderColor(Color::srgba(0.78, 0.57, 0.92, 0.95)),
                SpeechBubble {
                    owner,
                    lifetime: 2.0,
                },
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(color[0], color[1], color[2])),
                ));
            })
            .id();
        walk.bubble = Some(id);
        walk.speech_cooldown = 1.5;
    }
}

pub fn update_speech_bubbles(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut SpeechBubble, &mut Node)>,
    chars: Query<&Transform, With<Character>>,
    camera: Query<(&Camera, &GlobalTransform), Without<Character>>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();
    let Ok((cam, cam_xform)) = camera.get_single() else {
        return;
    };
    for (e, mut bub, mut node) in bubbles.iter_mut() {
        bub.lifetime -= dt;
        if bub.lifetime <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        let Ok(t) = chars.get(bub.owner) else {
            commands.entity(e).despawn_recursive();
            continue;
        };
        let head = t.translation + Vec3::new(0.0, 1.55, 0.0);
        if let Ok(s) = cam.world_to_viewport(cam_xform, head) {
            node.left = Val::Px(s.x - 48.0);
            node.top = Val::Px(s.y - 40.0);
        } else {
            node.left = Val::Px(-9999.0);
            node.top = Val::Px(-9999.0);
        }
    }
}
