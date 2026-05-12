package com.noahsark.effect;

import net.minecraft.entity.LivingEntity;
import net.minecraft.entity.effect.StatusEffect;
import net.minecraft.entity.effect.StatusEffectCategory;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.entity.effect.StatusEffects;
import net.minecraft.server.world.ServerWorld;

public class SlurpyEffect extends StatusEffect {
    public SlurpyEffect() {
        super(StatusEffectCategory.HARMFUL, 0x6600cc);
    }

    @Override
    public boolean applyUpdateEffect(ServerWorld world, LivingEntity entity, int amplifier) {
        entity.addStatusEffect(new StatusEffectInstance(StatusEffects.SLOWNESS, 40, 1, true, false, false));
        entity.addStatusEffect(new StatusEffectInstance(StatusEffects.WITHER,   40, 0, true, false, false));
        return true;
    }

    @Override
    public boolean canApplyUpdateEffect(int duration, int amplifier) {
        return true;
    }
}
