package com.noahsark.client;

import com.noahsark.NoahArkMod;
import com.noahsark.client.renderer.NullLightningRenderer;
import com.noahsark.client.renderer.SlurplingRenderer;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.render.fluid.v1.FluidRenderHandlerRegistry;
import net.fabricmc.fabric.api.client.render.fluid.v1.SimpleFluidRenderHandler;
import net.fabricmc.fabric.api.client.rendering.v1.EntityRendererRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;

public class NoahArkModClient implements ClientModInitializer {
    @Override
    public void onInitializeClient() {
        FluidRenderHandlerRegistry.INSTANCE.register(
            NoahArkMod.STILL_SLURP,
            NoahArkMod.FLOWING_SLURP,
            new SimpleFluidRenderHandler(
                SimpleFluidRenderHandler.WATER_STILL,
                SimpleFluidRenderHandler.WATER_FLOWING,
                0x6600cc
            )
        );
        EntityRendererRegistry.register(NoahArkMod.SLURPLING, SlurplingRenderer::new);
        //noinspection unchecked,rawtypes
        EntityRendererRegistry.register(
            (net.minecraft.entity.EntityType) NoahArkMod.NULL_LIGHTNING,
            (net.minecraft.client.render.entity.EntityRendererFactory) (net.minecraft.client.render.entity.EntityRendererFactory<net.minecraft.entity.LightningEntity>) NullLightningRenderer::new);
        HudRenderCallback.EVENT.register(new SlurpyHudOverlay());
    }
}
