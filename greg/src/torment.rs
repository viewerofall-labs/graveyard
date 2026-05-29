use bevy::prelude::*;

use crate::character::{random_position, Character, CharacterKind, WalkState, CHAR_HEIGHT};

#[derive(Resource, Default)]
pub struct Torment {
    pub value: f32,
    pub hits: u32,
    pub elapsed: f32,
    pub game_over: bool,
}

#[derive(Resource, Default)]
pub struct Combo {
    pub count: u32,
    pub timer: f32,
    pub best: u32,
}

#[derive(Resource, Default)]
pub struct HitStop {
    pub remaining: f32,
}

#[derive(Event, Clone, Copy)]
pub enum TormentEvent {
    Punch,
    Smite,
    Jail,
    Trip,
    Lava,
    Pit,
    Bounce,
    Yeet,
    Piano,
}

#[derive(Component)]
pub struct TormentBarFill;

#[derive(Component)]
pub struct TormentBar;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct GameOverOverlay;

const COMBO_WINDOW: f32 = 1.5;

pub fn drain_torment_events(
    mut ev: EventReader<TormentEvent>,
    mut torment: ResMut<Torment>,
    mut combo: ResMut<Combo>,
    mut hit_stop: ResMut<HitStop>,
) {
    for e in ev.read() {
        if torment.game_over {
            continue;
        }
        let (amt, big) = match *e {
            TormentEvent::Punch => (0.018, false),
            TormentEvent::Smite => (0.10, true),
            TormentEvent::Jail => (0.04, false),
            TormentEvent::Trip => (0.025, false),
            TormentEvent::Lava => (0.10, true),
            TormentEvent::Pit => (0.05, false),
            TormentEvent::Bounce => (0.02, false),
            TormentEvent::Yeet => (0.15, true),
            TormentEvent::Piano => (0.12, true),
        };
        torment.value = (torment.value + amt).min(1.0);
        torment.hits += 1;
        combo.count += 1;
        combo.timer = COMBO_WINDOW;
        if combo.count > combo.best {
            combo.best = combo.count;
        }
        if big {
            hit_stop.remaining = hit_stop.remaining.max(0.09);
        }
        if torment.value >= 1.0 {
            torment.game_over = true;
        }
    }
}

pub fn tick_torment_clock(mut t: ResMut<Torment>, real: Res<Time<Real>>) {
    if !t.game_over {
        t.elapsed += real.delta().as_secs_f32();
    }
}

pub fn tick_combo(mut combo: ResMut<Combo>, real: Res<Time<Real>>) {
    if combo.timer > 0.0 {
        combo.timer = (combo.timer - real.delta().as_secs_f32()).max(0.0);
        if combo.timer <= 0.0 {
            combo.count = 0;
        }
    }
}

pub fn spawn_torment_ui(commands: &mut Commands) {
    let fill = commands
        .spawn((
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.78, 0.20, 0.40)),
            TormentBarFill,
        ))
        .id();
    let parent = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(34.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                margin: UiRect::horizontal(Val::Auto),
                width: Val::Px(320.0),
                height: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.0, 0.06, 0.85)),
            BorderColor(Color::srgba(0.78, 0.57, 0.92, 0.85)),
            TormentBar,
        ))
        .id();
    commands.entity(parent).add_child(fill);

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            margin: UiRect::horizontal(Val::Auto),
            width: Val::Px(120.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new("TORMENT"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.57, 0.92)),
            ));
        });

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(58.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            margin: UiRect::horizontal(Val::Auto),
            width: Val::Px(160.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.9, 0.78, 1.0)),
                ComboText,
            ));
        });
}

pub fn update_torment_ui(torment: Res<Torment>, mut fills: Query<&mut Node, With<TormentBarFill>>) {
    for mut fill in fills.iter_mut() {
        fill.width = Val::Percent(torment.value.clamp(0.0, 1.0) * 100.0);
    }
}

pub fn update_combo_ui(
    combo: Res<Combo>,
    mut texts: Query<(&mut Text, &mut TextColor), With<ComboText>>,
) {
    for (mut text, mut color) in texts.iter_mut() {
        if combo.count >= 2 {
            text.0 = format!("x{}", combo.count);
            let a = (combo.timer / COMBO_WINDOW).clamp(0.0, 1.0);
            color.0 = Color::srgba(0.0, 0.9, 0.78, a);
        } else if !text.0.is_empty() {
            text.0.clear();
        }
    }
}

pub fn show_gameover(
    mut commands: Commands,
    torment: Res<Torment>,
    combo: Res<Combo>,
    overlays: Query<Entity, With<GameOverOverlay>>,
) {
    if !torment.game_over {
        return;
    }
    if !overlays.is_empty() {
        return;
    }
    let minutes = (torment.elapsed / 60.0) as u32;
    let seconds = (torment.elapsed % 60.0) as u32;
    let time_str = format!("{}m {}s", minutes, seconds);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.0, 0.06, 0.94)),
            GameOverOverlay,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("GREG HAS BEEN BULLIED"),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.57, 0.92)),
            ));
            p.spawn((
                Text::new(format!("{} hits delivered", torment.hits)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
            p.spawn((
                Text::new(format!("longest combo: x{}", combo.best.max(combo.count))),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
            p.spawn((
                Text::new(format!("torment time: {}", time_str)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
            p.spawn((
                Text::new("press SPACE to play again"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.9, 0.78)),
            ));
        });
}

pub fn reset_on_space(
    mut commands: Commands,
    mut torment: ResMut<Torment>,
    mut combo: ResMut<Combo>,
    mut hit_stop: ResMut<HitStop>,
    keys: Res<ButtonInput<KeyCode>>,
    overlays: Query<Entity, With<GameOverOverlay>>,
    mut chars: Query<(&mut Transform, &mut WalkState, &Character)>,
) {
    if !torment.game_over {
        return;
    }
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    torment.value = 0.0;
    torment.hits = 0;
    torment.elapsed = 0.0;
    torment.game_over = false;
    combo.count = 0;
    combo.timer = 0.0;
    combo.best = 0;
    hit_stop.remaining = 0.0;

    for (mut t, mut w, ch) in chars.iter_mut() {
        let pos = match ch.kind {
            CharacterKind::Greg => Vec3::new(-3.0, CHAR_HEIGHT, 0.0),
            CharacterKind::Fred => Vec3::new(3.0, CHAR_HEIGHT, 0.0),
        };
        t.translation = pos;
        t.rotation = Quat::IDENTITY;
        w.hp = 1.0;
        w.anger = if matches!(ch.kind, CharacterKind::Fred) {
            0.25
        } else {
            0.0
        };
        w.bruise = 0.0;
        w.smited = false;
        w.smite_timer = 0.0;
        w.dismembered = false;
        w.held = false;
        w.fallen_remaining = 0.0;
        w.dying_timer = 0.0;
        w.bounce_remaining = 0.0;
        w.in_pit = 0.0;
        w.jailed_remaining = 0.0;
        w.ascending_remaining = 0.0;
        w.is_staring = false;
        w.target = random_position();
        w.target_timer = 0.5;
        t.scale = Vec3::ONE;
    }

    crate::save::delete_save();

    for e in overlays.iter() {
        commands.entity(e).despawn_recursive();
    }
}
