use std::path::PathBuf;

use bevy::prelude::*;

use crate::cache;
use crate::character::{random_position, Character, CharacterKind, WalkState};
use crate::torment::{Combo, Torment};

const SAVE_MAGIC: u32 = 0x47524547; // "GREG"
const SAVE_VERSION: u32 = 1;

pub struct CharacterSave {
    pub pos: [f32; 3],
    pub hp: f32,
    pub anger: f32,
    pub bruise: f32,
}

pub struct GameSave {
    pub torment: f32,
    pub torment_hits: u32,
    pub torment_elapsed: f32,
    pub combo_best: u32,
    pub greg: CharacterSave,
    pub fred: CharacterSave,
}

#[derive(Resource)]
pub struct PendingSave(pub GameSave);

#[derive(Resource, Default)]
pub struct AutoSaveTimer(pub f32);

pub fn save_path() -> PathBuf {
    cache::cache_dir().join("save.bin")
}

fn read_f32(bytes: &[u8], o: &mut usize) -> Option<f32> {
    let slice = bytes.get(*o..*o + 4)?;
    *o += 4;
    Some(f32::from_le_bytes(slice.try_into().ok()?))
}

fn read_u32(bytes: &[u8], o: &mut usize) -> Option<u32> {
    let slice = bytes.get(*o..*o + 4)?;
    *o += 4;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

fn read_char(bytes: &[u8], o: &mut usize) -> Option<CharacterSave> {
    Some(CharacterSave {
        pos: [
            read_f32(bytes, o)?,
            read_f32(bytes, o)?,
            read_f32(bytes, o)?,
        ],
        hp: read_f32(bytes, o)?,
        anger: read_f32(bytes, o)?,
        bruise: read_f32(bytes, o)?,
    })
}

impl GameSave {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(80);
        b.extend(SAVE_MAGIC.to_le_bytes());
        b.extend(SAVE_VERSION.to_le_bytes());
        b.extend(self.torment.to_le_bytes());
        b.extend(self.torment_hits.to_le_bytes());
        b.extend(self.torment_elapsed.to_le_bytes());
        b.extend(self.combo_best.to_le_bytes());
        for c in [&self.greg, &self.fred] {
            for x in &c.pos {
                b.extend(x.to_le_bytes());
            }
            b.extend(c.hp.to_le_bytes());
            b.extend(c.anger.to_le_bytes());
            b.extend(c.bruise.to_le_bytes());
        }
        b
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut o = 0;
        let magic = read_u32(bytes, &mut o)?;
        let version = read_u32(bytes, &mut o)?;
        if magic != SAVE_MAGIC || version != SAVE_VERSION {
            return None;
        }
        let torment = read_f32(bytes, &mut o)?;
        let torment_hits = read_u32(bytes, &mut o)?;
        let torment_elapsed = read_f32(bytes, &mut o)?;
        let combo_best = read_u32(bytes, &mut o)?;
        let greg = read_char(bytes, &mut o)?;
        let fred = read_char(bytes, &mut o)?;
        Some(GameSave {
            torment,
            torment_hits,
            torment_elapsed,
            combo_best,
            greg,
            fred,
        })
    }
}

pub fn load_save() -> Option<GameSave> {
    let bytes = std::fs::read(save_path()).ok()?;
    GameSave::from_bytes(&bytes)
}

pub fn delete_save() {
    let _ = std::fs::remove_file(save_path());
}

pub fn write_save(save: &GameSave) {
    let _ = std::fs::write(save_path(), save.to_bytes());
}

pub fn apply_save_state(
    mut commands: Commands,
    save: Option<Res<PendingSave>>,
    mut chars: Query<(&mut Transform, &mut WalkState, &Character)>,
    mut torment: ResMut<Torment>,
    mut combo: ResMut<Combo>,
) {
    let Some(save) = save else {
        return;
    };
    for (mut t, mut w, ch) in chars.iter_mut() {
        let c = match ch.kind {
            CharacterKind::Greg => &save.0.greg,
            CharacterKind::Fred => &save.0.fred,
        };
        t.translation = Vec3::from(c.pos);
        w.hp = c.hp;
        w.anger = c.anger;
        w.bruise = c.bruise;
        w.target = random_position();
        w.target_timer = 0.5;
    }
    torment.value = save.0.torment;
    torment.hits = save.0.torment_hits;
    torment.elapsed = save.0.torment_elapsed;
    combo.best = save.0.combo_best;
    commands.remove_resource::<PendingSave>();
}

pub fn autosave_system(
    mut timer: ResMut<AutoSaveTimer>,
    real: Res<Time<Real>>,
    chars: Query<(&Transform, &WalkState, &Character)>,
    torment: Res<Torment>,
    combo: Res<Combo>,
) {
    if torment.game_over {
        return;
    }
    timer.0 += real.delta().as_secs_f32();
    if timer.0 < 10.0 {
        return;
    }
    // Skip save if anyone is mid-transient (ascending/falling/etc) — wait til next tick
    if chars
        .iter()
        .any(|(_, w, _)| w.ascending_remaining > 0.0 || w.in_pit > 0.0 || w.bounce_remaining > 0.0)
    {
        return;
    }
    timer.0 = 0.0;

    let mut greg = None;
    let mut fred = None;
    for (t, w, ch) in chars.iter() {
        let cs = CharacterSave {
            pos: [t.translation.x, t.translation.y, t.translation.z],
            hp: w.hp,
            anger: w.anger,
            bruise: w.bruise,
        };
        match ch.kind {
            CharacterKind::Greg => greg = Some(cs),
            CharacterKind::Fred => fred = Some(cs),
        }
    }
    let (Some(greg), Some(fred)) = (greg, fred) else {
        return;
    };
    write_save(&GameSave {
        torment: torment.value,
        torment_hits: torment.hits,
        torment_elapsed: torment.elapsed,
        combo_best: combo.best,
        greg,
        fred,
    });
}
