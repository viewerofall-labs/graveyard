package com.noahsark.mixin;

import com.noahsark.fluid.SlurpFluid;
import net.minecraft.entity.vehicle.AbstractBoatEntity;
import net.minecraft.util.math.BlockPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(AbstractBoatEntity.class)
public abstract class BoatSlurpMixin {
    @Inject(method = "tick", at = @At("TAIL"))
    private void sinkInSlurp(CallbackInfo ci) {
        AbstractBoatEntity self = (AbstractBoatEntity)(Object)this;
        if (self.getWorld().isClient()) return;
        BlockPos pos = self.getBlockPos();
        for (int dy = 0; dy <= 1; dy++) {
            var fluid = self.getWorld().getFluidState(pos.up(dy));
            if (fluid.getFluid() instanceof SlurpFluid) {
                self.setVelocity(self.getVelocity().add(0, -0.08, 0));
                return;
            }
        }
    }
}
