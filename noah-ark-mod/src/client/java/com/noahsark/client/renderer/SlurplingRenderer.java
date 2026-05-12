package com.noahsark.client.renderer;

import net.minecraft.client.render.entity.EntityRendererFactory;
import net.minecraft.client.render.entity.ZombieEntityRenderer;
import net.minecraft.client.render.entity.state.ZombieEntityRenderState;
import net.minecraft.util.Identifier;

public class SlurplingRenderer extends ZombieEntityRenderer {
    private static final Identifier TEXTURE = Identifier.of("noahsark", "textures/entity/slurpling.png");

    public SlurplingRenderer(EntityRendererFactory.Context context) {
        super(context);
    }

    @Override
    public Identifier getTexture(ZombieEntityRenderState state) {
        return TEXTURE;
    }
}
