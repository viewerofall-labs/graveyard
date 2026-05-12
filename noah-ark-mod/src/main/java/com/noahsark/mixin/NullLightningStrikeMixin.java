package com.noahsark.mixin;

import com.noahsark.entity.NullLightningEntity;
import net.minecraft.entity.Entity;
import net.minecraft.entity.LightningEntity;
import net.minecraft.entity.LivingEntity;
import net.minecraft.server.world.ServerWorld;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Entity.class)
public abstract class NullLightningStrikeMixin {
    @Inject(method = "onStruckByLightning", at = @At("HEAD"), cancellable = true)
    private void nullLightningStrike(ServerWorld world, LightningEntity lightning, CallbackInfo ci) {
        if (!(lightning instanceof NullLightningEntity)) return;
        Entity self = (Entity)(Object)this;
        if (self instanceof LivingEntity living) {
            living.damage(world, world.getDamageSources().lightningBolt(), 6.0f);
        }
        self.setOnFireFor(8f);
        ci.cancel();
    }
}
