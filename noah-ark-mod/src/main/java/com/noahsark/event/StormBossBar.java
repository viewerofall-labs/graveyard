package com.noahsark.event;

import net.minecraft.entity.boss.BossBar;
import net.minecraft.entity.boss.ServerBossBar;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.network.ServerPlayerEntity;
import net.minecraft.text.Text;

public class StormBossBar {
    private static final int FLOOD_START  = StormManager.FLOOD_START_TICKS;
    private static final int TOTAL_TICKS  = StormManager.TOTAL_TICKS;

    private final ServerBossBar bar;

    public StormBossBar() {
        bar = new ServerBossBar(
            Text.literal("The flood is coming..."),
            BossBar.Color.PURPLE,
            BossBar.Style.PROGRESS
        );
        bar.setDarkenSky(true);
    }

    public void addPlayers(MinecraftServer server) {
        for (ServerPlayerEntity player : server.getPlayerManager().getPlayerList()) {
            if (!bar.getPlayers().contains(player)) bar.addPlayer(player);
        }
    }

    public void update(int tick) {
        if (tick < FLOOD_START) {
            int remaining = FLOOD_START - tick;
            int minutes = remaining / 1200;
            int seconds = (remaining % 1200) / 20;
            bar.setName(Text.literal(String.format("Flood begins in %d:%02d — Build your ark!", minutes, seconds)));
            bar.setPercent((float) remaining / FLOOD_START);
            bar.setColor(BossBar.Color.YELLOW);
        } else {
            int elapsed   = tick - FLOOD_START;
            int remaining = TOTAL_TICKS - tick;
            int minutes   = Math.max(0, remaining / 1200);
            int seconds   = Math.max(0, (remaining % 1200) / 20);
            bar.setName(Text.literal(String.format("Slurp flood rising — %d:%02d remaining", minutes, seconds)));
            bar.setPercent(Math.max(0f, (float)(TOTAL_TICKS - tick) / (TOTAL_TICKS - FLOOD_START)));
            bar.setColor(BossBar.Color.PURPLE);
        }
    }

    public void onPlayerJoin(ServerPlayerEntity player) {
        bar.addPlayer(player);
    }

    public void remove() {
        bar.clearPlayers();
    }
}
