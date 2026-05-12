package com.noahsark.mixin;

import com.noahsark.NoahArkMod;
import com.noahsark.entity.NullLightningEntity;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.LightningEntity;
import net.minecraft.entity.SpawnReason;
import net.minecraft.server.world.ServerWorld;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(LightningEntity.class)
public abstract class LightningInterceptMixin {
    @Inject(method = "tick", at = @At("HEAD"), cancellable = true)
    private void interceptNaturalLightning(CallbackInfo ci) {
        LightningEntity self = (LightningEntity) (Object) this;
        if (self.getWorld().isClient()) return;
        if (self instanceof NullLightningEntity) return;
        if (self.age != 0) return;
        if (self.getWorld().getRandom().nextFloat() >= 0.65f) return;

        ServerWorld world = (ServerWorld) self.getWorld();
        NullLightningEntity nullBolt = NoahArkMod.NULL_LIGHTNING.create(world, SpawnReason.EVENT);
        if (nullBolt != null) {
            nullBolt.setPosition(self.getX(), self.getY(), self.getZ());
            world.spawnEntity(nullBolt);
        }
        self.discard();
        ci.cancel();
    }
}
