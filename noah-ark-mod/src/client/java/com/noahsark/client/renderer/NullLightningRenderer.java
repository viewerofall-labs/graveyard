package com.noahsark.client.renderer;

import net.minecraft.client.render.entity.EntityRendererFactory;
import net.minecraft.client.render.entity.LightningEntityRenderer;

public class NullLightningRenderer extends LightningEntityRenderer {
    public NullLightningRenderer(EntityRendererFactory.Context ctx) {
        super(ctx);
    }
}
