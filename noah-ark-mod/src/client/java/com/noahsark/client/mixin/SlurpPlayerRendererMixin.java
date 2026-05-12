package com.noahsark.client.mixin;

import com.noahsark.NoahArkMod;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.entity.PlayerEntityRenderer;
import net.minecraft.client.render.entity.state.PlayerEntityRenderState;
import net.minecraft.client.util.SkinTextures;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(PlayerEntityRenderer.class)
public abstract class SlurpPlayerRendererMixin {
    private static final Identifier SLURP_SKIN =
        Identifier.of("noahsark", "textures/entity/slurp_skin.png");

    @Inject(method = "updateRenderState", at = @At("TAIL"))
    private void swapSlurpSkin(AbstractClientPlayerEntity player,
                                PlayerEntityRenderState state,
                                float tickDelta, CallbackInfo ci) {
        if (!player.hasStatusEffect(NoahArkMod.SLURPY)) return;
        var ex = state.skinTextures;
        state.skinTextures = new SkinTextures(
            SLURP_SKIN, null,
            ex.capeTexture(), ex.elytraTexture(),
            ex.model(), ex.secure()
        );
    }
}
