package com.noahsark.mixin;

import com.noahsark.event.LockedHeartTracker;
import net.minecraft.entity.LivingEntity;
import net.minecraft.server.network.ServerPlayerEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

@Mixin(LivingEntity.class)
public abstract class LockedHeartHealMixin {
    @ModifyVariable(method = "heal", at = @At("HEAD"), argsOnly = true, index = 1)
    private float capHealForLockedHearts(float amount) {
        LivingEntity self = (LivingEntity)(Object)this;
        if (!(self instanceof ServerPlayerEntity player)) return amount;
        int locked = LockedHeartTracker.getLocked(player);
        if (locked <= 0) return amount;
        float cap = self.getMaxHealth() - (locked * 2f);
        float current = self.getHealth();
        if (current >= cap) return 0f;
        return Math.min(amount, cap - current);
    }
}
