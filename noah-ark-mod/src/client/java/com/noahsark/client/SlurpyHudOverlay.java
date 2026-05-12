package com.noahsark.client;

import com.noahsark.event.LockedHeartTracker;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.render.RenderLayer;
import net.minecraft.client.render.RenderTickCounter;
import net.minecraft.util.Identifier;

public final class SlurpyHudOverlay implements HudRenderCallback {
    // Vanilla withered full heart sprite
    private static final Identifier WITHERED_FULL =
        Identifier.ofVanilla("hud/heart/withered_full");
    // Vanilla heart container (background)
    private static final Identifier CONTAINER =
        Identifier.ofVanilla("hud/heart/container");

    @Override
    public void onHudRender(DrawContext ctx, RenderTickCounter tickCounter) {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc.player == null || mc.options.hudHidden) return;

        // Read locked hearts from client-side scoreboard
        var scoreboard = mc.player.getScoreboard();
        var obj = scoreboard.getNullableObjective(LockedHeartTracker.OBJECTIVE);
        if (obj == null) return;
        var scoreEntry = scoreboard.getScore(mc.player, obj);
        int locked = scoreEntry != null ? scoreEntry.getScore() : 0;
        if (locked <= 0) return;

        int screenW = ctx.getScaledWindowWidth();
        int screenH = ctx.getScaledWindowHeight();

        // Vanilla heart bar: x=9, y=screenH-49 (with armor) or screenH-39 (no armor)
        // Use screenH-49 to be safe — matches vanilla offset
        int barX = 9;
        int barY = screenH - 49;

        // Hearts are 9px wide. Total 10 hearts displayed left-to-right.
        // Locked hearts start from slot (10 - locked) going right.
        int startSlot = 10 - locked;
        for (int i = startSlot; i < 10; i++) {
            int x = barX + i * 8;
            // Draw container background first
            ctx.drawGuiTexture(RenderLayer::getGuiTextured, CONTAINER, x, barY, 9, 9);
            // Draw withered full heart on top
            ctx.drawGuiTexture(RenderLayer::getGuiTextured, WITHERED_FULL, x, barY, 9, 9);
        }
    }
}
