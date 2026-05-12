package com.noahsark.event;

import com.noahsark.NoahArkMod;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.network.ServerPlayerEntity;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.text.Text;
import net.minecraft.util.math.BlockPos;

import java.util.HashSet;
import java.util.Set;
import java.util.UUID;

public final class CurseTracker {
    public static final int CURSE_Y = 55;
    private static final int CURSE_RADIUS = 24;

    private static boolean worldCursed = false;
    private static final Set<UUID> corrupted = new HashSet<>();

    private CurseTracker() {}

    public static boolean isWorldCursed() { return worldCursed; }

    public static void markCorrupted(MinecraftServer server, UUID uuid) {
        corrupted.add(uuid);
        tryTriggerCurse(server);
    }

    private static void tryTriggerCurse(MinecraftServer server) {
        if (worldCursed) return;
        var players = server.getPlayerManager().getPlayerList();
        if (players.isEmpty()) return;
        if (NoahArkMod.stormManager == null || !NoahArkMod.stormManager.isFlooded()) return;
        for (ServerPlayerEntity p : players) {
            if (!corrupted.contains(p.getUuid())) return;
        }
        if (!NoahArkMod.placedArks.isEmpty() && anyPlayerOnArk(server)) return;

        worldCursed = true;
        for (ServerPlayerEntity p : players) {
            applyCurseEffect(p);
            p.sendMessage(Text.literal("§5[Noah's Ark] §4The world is cursed. The slurp rises forever."), false);
        }
    }

    private static boolean anyPlayerOnArk(MinecraftServer server) {
        for (var ap : NoahArkMod.placedArks) {
            for (ServerPlayerEntity p : server.getPlayerManager().getPlayerList()) {
                var o = ap.origin;
                if (Math.abs(p.getBlockX() - o.getX()) <= 20 &&
                    Math.abs(p.getBlockZ() - o.getZ()) <= 20 &&
                    p.getY() >= o.getY() - 1 && p.getY() <= o.getY() + 12) return true;
            }
        }
        return false;
    }

    public static void applyCurseEffect(ServerPlayerEntity player) {
        player.addStatusEffect(new StatusEffectInstance(NoahArkMod.WORLD_CURSED, Integer.MAX_VALUE, 0, false, true));
    }

    public static void tick(ServerWorld world) {
        if (!worldCursed) return;
        var slurpState = NoahArkMod.SLURP_BLOCK.getDefaultState();
        for (ServerPlayerEntity player : world.getPlayers()) {
            if (!player.hasStatusEffect(NoahArkMod.WORLD_CURSED)) applyCurseEffect(player);
            int px = player.getBlockX(), pz = player.getBlockZ();
            for (int x = px - CURSE_RADIUS; x <= px + CURSE_RADIUS; x++) {
                for (int z = pz - CURSE_RADIUS; z <= pz + CURSE_RADIUS; z++) {
                    BlockPos pos = new BlockPos(x, CURSE_Y, z);
                    if (world.getBlockState(pos).isAir()) {
                        world.setBlockState(pos, slurpState);
                    }
                }
            }
        }
    }

    public static void reset() {
        worldCursed = false;
        corrupted.clear();
    }
}
