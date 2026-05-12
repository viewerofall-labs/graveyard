package com.noahsark.event;

import com.noahsark.NoahArkMod;
import com.noahsark.entity.SlurplingEntity;
import com.noahsark.world.NoahsArkPlacer;

import net.minecraft.block.Blocks;
import net.minecraft.entity.SpawnReason;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.network.ServerPlayerEntity;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.text.Text;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;

public class StormManager {
    // Warning phase: 18000 ticks (15 min). Flood phase: 6000 ticks (5 min).
    public static final int FLOOD_START_TICKS = 18000;
    public static final int TOTAL_TICKS       = 24000;

    private static final int SEA_LEVEL          = 60;
    private static final int FLOOD_RISE_INTERVAL = 100; // 1 block per 5s
    private static final int FLOOD_RADIUS        = 80;  // 5-chunk radius = 10-chunk diameter

    private int tickCounter = 0;
    private boolean active   = true;
    private boolean flooded  = false;
    private boolean receding = false;
    private int recessingY   = 0;

    private final StormBossBar bossBar;

    public StormManager() {
        bossBar = new StormBossBar();
    }

    public void tick(ServerWorld world) {
        if (!active) return;

        tickCounter++;

        bossBar.addPlayers(world.getServer());
        bossBar.update(tickCounter);

        if (tickCounter % 200 == 0) refreshWeather(world);

        // Recession phase: drain flood from top down, 2 layers per interval
        if (receding) {
            if (tickCounter % FLOOD_RISE_INTERVAL == 0) {
                removeFloodLayer(world, recessingY);
                removeFloodLayer(world, recessingY - 1);
                recessingY -= 2;
                if (recessingY < SEA_LEVEL) {
                    active = false;
                    receding = false;
                    bossBar.remove();
                    cleanFlood(world);
                    broadcastTitle(world.getServer(), "The Storm Has Passed", "The world is clean once more.");
                    CurseTracker.reset();
                }
            }
            return;
        }

        if (tickCounter < FLOOD_START_TICKS) {
            // Warning phase: heavy rain + occasional slurp blobs to collect
            if (tickCounter == 1) {
                refreshWeather(world);
                broadcastTitle(world.getServer(),
                    "The Storm Begins",
                    "Build your ark before the flood — 15 minutes remain");
            }
            if (tickCounter % 600 == 0) spawnSlurpBlobs(world);
        } else {
            // Flood phase
            if (!flooded) {
                flooded = true;
                refreshWeather(world);
                broadcastTitle(world.getServer(),
                    "The Flood Has Come!",
                    "Slurp juice is rising — get to higher ground!");
            }

            if (tickCounter % 40 == 0) {
                int floodY = getFloodY();
                spawnFlood(world, floodY);
            }

            if (tickCounter % 300 == 0) {
                spawnSlurplings(world);
            }

            // Rise and drift arks every FLOOD_RISE_INTERVAL ticks (same rate flood rises)
            if (tickCounter % FLOOD_RISE_INTERVAL == 0) {
                NoahsArkPlacer.tickArks(world, getFloodY(), tickCounter);
            }
        }

        if (tickCounter >= TOTAL_TICKS) {
            if (CurseTracker.isWorldCursed()) {
                // Cursed world — storm ends but flood stays forever
                active = false;
                bossBar.remove();
            } else {
                receding = true;
                recessingY = Math.min(getFloodY(), world.getTopYInclusive());
                broadcastTitle(world.getServer(), "The Storm Is Lifting", "The waters begin to recede...");
            }
        }
    }

    private void spawnSlurplings(ServerWorld world) {
        int floodY = getFloodY();
        for (ServerPlayerEntity player : world.getPlayers()) {
            int px = player.getBlockX(), pz = player.getBlockZ();
            for (int attempt = 0; attempt < 2; attempt++) {
                int x = px + world.getRandom().nextBetween(-16, 16);
                int z = pz + world.getRandom().nextBetween(-16, 16);
                var fluid = world.getFluidState(new BlockPos(x, floodY, z)).getFluid();
                if (fluid != NoahArkMod.STILL_SLURP && fluid != NoahArkMod.FLOWING_SLURP) continue;
                BlockPos surface = world.getTopPosition(Heightmap.Type.WORLD_SURFACE, new BlockPos(x, 0, z));
                BlockPos spawnPos = surface.getY() > floodY ? surface : new BlockPos(x, floodY + 1, z);
                SlurplingEntity mob = NoahArkMod.SLURPLING.create(world, SpawnReason.EVENT);
                if (mob == null) continue;
                mob.setPosition(spawnPos.getX() + 0.5, spawnPos.getY(), spawnPos.getZ() + 0.5);
                world.spawnEntity(mob);
            }
        }
    }

    private void spawnSlurpBlobs(ServerWorld world) {
        var slurpState = NoahArkMod.SLURP_BLOCK.getDefaultState();
        for (ServerPlayerEntity player : world.getPlayers()) {
            int px = player.getBlockX(), pz = player.getBlockZ();
            int blobs = 2 + world.getRandom().nextInt(2); // 2-3 blobs
            for (int i = 0; i < blobs; i++) {
                int x = px + world.getRandom().nextBetween(-32, 32);
                int z = pz + world.getRandom().nextBetween(-32, 32);
                BlockPos top = world.getTopPosition(Heightmap.Type.WORLD_SURFACE, new BlockPos(x, 0, z));
                if (world.getBlockState(top).isAir()) {
                    world.setBlockState(top, slurpState);
                }
            }
        }
    }

    private void spawnFlood(ServerWorld world, int floodY) {
        var slurpState = NoahArkMod.SLURP_BLOCK.getDefaultState();
        for (ServerPlayerEntity player : world.getPlayers()) {
            int px = player.getBlockX();
            int pz = player.getBlockZ();

            // Fill entire 10-chunk diameter area flat at floodY
            for (int x = px - FLOOD_RADIUS; x <= px + FLOOD_RADIUS; x++) {
                for (int z = pz - FLOOD_RADIUS; z <= pz + FLOOD_RADIUS; z++) {
                    BlockPos floodPos = new BlockPos(x, floodY, z);
                    var state = world.getBlockState(floodPos);
                    if (state.isAir() || state.isOf(Blocks.WATER)) {
                        world.setBlockState(floodPos, slurpState);
                    }
                }
            }

            // Slurp rain from the sky for areas above flood level
            float progress = (float)(tickCounter - FLOOD_START_TICKS) / (TOTAL_TICKS - FLOOD_START_TICKS);
            int skyAttempts = 4 + (int)(progress * 8);
            for (int i = 0; i < skyAttempts; i++) {
                int x = px + world.getRandom().nextBetween(-FLOOD_RADIUS, FLOOD_RADIUS);
                int z = pz + world.getRandom().nextBetween(-FLOOD_RADIUS, FLOOD_RADIUS);
                BlockPos top = world.getTopPosition(Heightmap.Type.WORLD_SURFACE, new BlockPos(x, 0, z));
                if (world.isSkyVisible(top) && top.getY() > floodY) {
                    world.setBlockState(top, slurpState);
                }
            }
        }
    }

    private void refreshWeather(ServerWorld world) {
        if (tickCounter < FLOOD_START_TICKS) {
            world.setWeather(0, 2400, true, false); // heavy rain pre-flood
        } else {
            world.setWeather(0, 2400, true, true);  // thunder during flood
        }
    }

    private void broadcastTitle(MinecraftServer server, String title, String subtitle) {
        for (ServerPlayerEntity p : server.getPlayerManager().getPlayerList()) {
            p.sendMessage(Text.literal("[Noah's Ark] " + title + (subtitle.isEmpty() ? "" : " — " + subtitle)), false);
        }
    }

    public int getFloodY() {
        if (tickCounter < FLOOD_START_TICKS) return SEA_LEVEL;
        return SEA_LEVEL + (tickCounter - FLOOD_START_TICKS) / FLOOD_RISE_INTERVAL;
    }

    public void skipToFlood() {
        tickCounter = FLOOD_START_TICKS;
        flooded = false; // let the transition logic fire next tick
    }

    public void onPlayerJoin(net.minecraft.server.network.ServerPlayerEntity player) {
        bossBar.onPlayerJoin(player);
    }

    /** Scan every loaded chunk and remove all slurp blocks, then clear weather. */
    public static int cleanFlood(ServerWorld world) {
        int removed = 0;
        var chunkManager = world.getChunkManager();
        int viewDist = world.getServer().getPlayerManager().getViewDistance() + 4;

        for (ServerPlayerEntity player : world.getPlayers()) {
            int pcx = player.getChunkPos().x;
            int pcz = player.getChunkPos().z;
            for (int cx = pcx - viewDist; cx <= pcx + viewDist; cx++) {
                for (int cz = pcz - viewDist; cz <= pcz + viewDist; cz++) {
                    if (!chunkManager.isChunkLoaded(cx, cz)) continue;
                    for (int lx = 0; lx < 16; lx++) {
                        for (int lz = 0; lz < 16; lz++) {
                            for (int y = world.getBottomY(); y <= world.getTopYInclusive(); y++) {
                                BlockPos pos = new BlockPos(cx * 16 + lx, y, cz * 16 + lz);
                                if (world.getBlockState(pos).isOf(NoahArkMod.SLURP_BLOCK)) {
                                    world.removeBlock(pos, false);
                                    removed++;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Clear weather
        world.setWeather(6000, 0, false, false);
        return removed;
    }

    public void forceEnd() {
        active = false;
        bossBar.remove();
    }

    private void removeFloodLayer(ServerWorld world, int y) {
        if (y < world.getBottomY() || y > world.getTopYInclusive()) return;
        int viewDist = world.getServer().getPlayerManager().getViewDistance() + 4;
        for (ServerPlayerEntity player : world.getPlayers()) {
            int pcx = player.getChunkPos().x;
            int pcz = player.getChunkPos().z;
            for (int cx = pcx - viewDist; cx <= pcx + viewDist; cx++) {
                for (int cz = pcz - viewDist; cz <= pcz + viewDist; cz++) {
                    if (!world.getChunkManager().isChunkLoaded(cx, cz)) continue;
                    for (int lx = 0; lx < 16; lx++) {
                        for (int lz = 0; lz < 16; lz++) {
                            BlockPos pos = new BlockPos(cx * 16 + lx, y, cz * 16 + lz);
                            if (world.getBlockState(pos).isOf(NoahArkMod.SLURP_BLOCK)) {
                                world.removeBlock(pos, false);
                            }
                        }
                    }
                }
            }
        }
    }

    public boolean isActive()   { return active; }
    public boolean isFlooded()  { return flooded; }
    public int getTickCounter() { return tickCounter; }
}
