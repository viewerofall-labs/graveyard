use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::character::{Character, WalkState, PUNCH_DUR};

#[derive(Clone, Copy)]
pub struct BoneRef {
    pub entity: Entity,
    pub bind: Quat,
}

#[derive(Default)]
pub struct BoneSet {
    pub upperarm_l: Option<BoneRef>,
    pub upperarm_r: Option<BoneRef>,
    pub lowerarm_l: Option<BoneRef>,
    pub lowerarm_r: Option<BoneRef>,
    pub thigh_l: Option<BoneRef>,
    pub thigh_r: Option<BoneRef>,
    pub calf_l: Option<BoneRef>,
    pub calf_r: Option<BoneRef>,
}

#[derive(Resource, Default)]
pub struct CharacterBones(pub HashMap<Entity, BoneSet>);

pub fn cache_bones(
    added: Query<(Entity, &Name, &Parent, &Transform), Added<Name>>,
    parents: Query<&Parent>,
    chars: Query<(), With<Character>>,
    mut cache: ResMut<CharacterBones>,
) {
    for (e, name, parent, t) in added.iter() {
        let mut cur = parent.get();
        let owner = loop {
            if chars.get(cur).is_ok() {
                break Some(cur);
            }
            match parents.get(cur) {
                Ok(p) => cur = p.get(),
                Err(_) => break None,
            }
        };
        let Some(owner) = owner else {
            continue;
        };
        let set = cache.0.entry(owner).or_default();
        let bone_ref = BoneRef {
            entity: e,
            bind: t.rotation,
        };
        match name.as_str() {
            "upperarm_l" => set.upperarm_l = Some(bone_ref),
            "upperarm_r" => set.upperarm_r = Some(bone_ref),
            "lowerarm_l" => set.lowerarm_l = Some(bone_ref),
            "lowerarm_r" => set.lowerarm_r = Some(bone_ref),
            "thigh_l" => set.thigh_l = Some(bone_ref),
            "thigh_r" => set.thigh_r = Some(bone_ref),
            "calf_l" => set.calf_l = Some(bone_ref),
            "calf_r" => set.calf_r = Some(bone_ref),
            _ => {}
        }
    }
}

pub fn animate_model_bones(
    chars: Query<(Entity, &WalkState), With<Character>>,
    cache: Res<CharacterBones>,
    mut bones: Query<&mut Transform>,
) {
    for (e, walk) in chars.iter() {
        let Some(set) = cache.0.get(&e) else {
            continue;
        };
        if !walk.has_model {
            continue;
        }

        let active = !walk.is_staring
            && !walk.held
            && walk.fallen_remaining == 0.0
            && !walk.dismembered
            && !walk.smited
            && walk.dying_timer <= 0.0
            && walk.bounce_remaining == 0.0
            && walk.in_pit == 0.0
            && walk.jailed_remaining <= 0.0;

        let amp = if active { 0.55 } else { 0.0 };
        let phase = walk.walk_phase;
        let swing = phase.sin() * amp;

        // Arms hang at sides (rotated down ~85deg around bone local Z) + swing on local X.
        // Bone local frames vary; we post-multiply so swings happen in bone-local space.
        let arms_down_l = Quat::from_rotation_z(-1.45);
        let arms_down_r = Quat::from_rotation_z(1.45);
        let arm_swing_l = Quat::from_rotation_x(-swing * 0.6);
        let arm_swing_r = Quat::from_rotation_x(swing * 0.6);

        // legs: opposite phase between left/right
        let leg_swing_l = Quat::from_rotation_x(swing);
        let leg_swing_r = Quat::from_rotation_x(-swing);
        // knee bend (always positive, follows |sin|)
        let knee_bend = phase.sin().max(0.0) * amp * 0.8;
        let knee_l = Quat::from_rotation_x(-knee_bend);
        let knee_r = Quat::from_rotation_x(-((-phase).sin().max(0.0) * amp * 0.8));

        // Punch override: right arm forward jab
        let punch_active = walk.punch_anim > 0.0;
        let punch_amount = if punch_active {
            let t = (PUNCH_DUR - walk.punch_anim) / PUNCH_DUR;
            let peak = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
            peak
        } else {
            0.0
        };
        let punch_arm = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * punch_amount);

        apply(&mut bones, set.upperarm_l, arms_down_l * arm_swing_l);
        apply(
            &mut bones,
            set.upperarm_r,
            if punch_active {
                arms_down_r * punch_arm
            } else {
                arms_down_r * arm_swing_r
            },
        );
        apply(&mut bones, set.thigh_l, leg_swing_l);
        apply(&mut bones, set.thigh_r, leg_swing_r);
        apply(&mut bones, set.calf_l, knee_l);
        apply(&mut bones, set.calf_r, knee_r);
    }
}

fn apply(bones: &mut Query<&mut Transform>, b: Option<BoneRef>, offset: Quat) {
    let Some(b) = b else { return };
    let Ok(mut t) = bones.get_mut(b.entity) else {
        return;
    };
    t.rotation = b.bind * offset;
}
