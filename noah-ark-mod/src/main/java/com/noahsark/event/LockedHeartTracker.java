package com.noahsark.event;

import net.minecraft.scoreboard.Scoreboard;
import net.minecraft.scoreboard.ScoreboardCriterion;
import net.minecraft.scoreboard.ScoreboardObjective;
import net.minecraft.server.network.ServerPlayerEntity;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.text.Text;

public final class LockedHeartTracker {
    public static final String OBJECTIVE = "ns_locked";
    private static final int MAX_LOCKED = 10;

    private LockedHeartTracker() {}

    private static ScoreboardObjective getOrCreateObjective(Scoreboard sb) {
        var obj = sb.getNullableObjective(OBJECTIVE);
        if (obj == null) {
            obj = sb.addObjective(OBJECTIVE, ScoreboardCriterion.DUMMY,
                Text.literal("Locked Hearts"),
                ScoreboardCriterion.RenderType.INTEGER, false, null);
        }
        return obj;
    }

    public static int getLocked(ServerPlayerEntity player) {
        var sb = player.server.getScoreboard();
        var obj = getOrCreateObjective(sb);
        var score = sb.getScore(player, obj);
        return score != null ? score.getScore() : 0;
    }

    public static void addLocked(ServerPlayerEntity player, int amount) {
        var sb = player.server.getScoreboard();
        var obj = getOrCreateObjective(sb);
        var score = sb.getOrCreateScore(player, obj);
        int newVal = Math.min(MAX_LOCKED, score.getScore() + amount);
        score.setScore(newVal);
        PlayerSubmersionTracker.drainShield(player, 10);
        if (newVal >= MAX_LOCKED) {
            player.server.getPlayerManager().broadcast(
                Text.literal("§5[Noah's Ark] §4" + player.getName().getString() + " has been fully corrupted by the slurp."),
                false);
            ServerWorld world = (ServerWorld) player.getWorld();
            player.damage(world, world.getDamageSources().magic(), Float.MAX_VALUE);
            CurseTracker.markCorrupted(player.server, player.getUuid());
        }
    }

    public static void clearLocked(ServerPlayerEntity player) {
        var sb = player.server.getScoreboard();
        var obj = getOrCreateObjective(sb);
        sb.getOrCreateScore(player, obj).setScore(0);
    }
}
