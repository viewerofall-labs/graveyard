use bevy::asset::AssetPlugin;
use bevy::prelude::*;

mod bone_anim;
mod cache;
mod camera;
mod character;
mod combat;
mod limbs;
mod save;
mod setup;
mod sfx;
mod smite;
mod torment;
mod ui;
mod world;

use bone_anim::{animate_model_bones, cache_bones, CharacterBones};
use camera::{orbit_camera, CameraShake};
use character::{
    update_anger_visual, update_bruise_visual, update_model_visibility, walk_characters,
};
use combat::{
    fred_attacks_greg, interact_with_characters, update_death, update_timescale, MouseAction,
    SlowMo,
};
use limbs::{animate_limbs, jail_input, update_dismembered_limbs, update_jail_cell};
use save::{
    apply_save_state, autosave_system, load_save, AutoSaveTimer, PendingSave,
};
use sfx::ensure_sfx;
use smite::{
    smite_input, spawn_smite_visuals, update_ash, update_lightning, update_smite, SmiteRequests,
};
use torment::{
    drain_torment_events, reset_on_space, show_gameover, tick_combo, tick_torment_clock,
    update_combo_ui, update_torment_ui, Combo, HitStop, Torment, TormentEvent,
};
use ui::{idle_speech, process_speech_queue, update_bars, update_speech_bubbles, SpeechQueue};
use world::{
    hazards_system, piano_input, trip_cubes_system, update_ascending, update_door, update_piano,
};

fn main() {
    let cache = cache::ensure_cache_seeded();
    ensure_sfx(&cache);

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: cache.to_string_lossy().into_owned(),
        ..default()
    }));

    if let Some(save) = load_save() {
        app.insert_resource(PendingSave(save));
    }

    app.init_resource::<MouseAction>()
        .init_resource::<CameraShake>()
        .init_resource::<SpeechQueue>()
        .init_resource::<SmiteRequests>()
        .init_resource::<SlowMo>()
        .init_resource::<CharacterBones>()
        .init_resource::<Torment>()
        .init_resource::<Combo>()
        .init_resource::<HitStop>()
        .init_resource::<AutoSaveTimer>()
        .add_event::<TormentEvent>()
        .add_systems(Startup, (setup::setup, apply_save_state).chain())
        .add_systems(
            Update,
            (
                reset_on_space,
                smite_input,
                spawn_smite_visuals,
                update_smite,
                jail_input,
                update_jail_cell,
                update_dismembered_limbs,
                piano_input,
                update_piano,
                update_ascending,
                fred_attacks_greg,
                interact_with_characters,
                hazards_system,
                update_death,
                trip_cubes_system,
                walk_characters,
                animate_limbs,
                drain_torment_events,
                process_speech_queue,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_ash,
                update_lightning,
                update_anger_visual,
                update_bruise_visual,
                update_model_visibility,
                cache_bones,
                animate_model_bones,
                update_door,
                update_bars,
                update_speech_bubbles,
                idle_speech,
                update_timescale,
                orbit_camera,
            ),
        )
        .add_systems(
            Update,
            (
                tick_torment_clock,
                tick_combo,
                update_torment_ui,
                update_combo_ui,
                show_gameover,
                autosave_system,
            ),
        )
        .run();
}
