package com.noahsark.event;

import com.noahsark.NoahArkMod;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.entity.EquipmentSlot;
import net.minecraft.entity.effect.StatusEffectInstance;
import net.minecraft.entity.effect.StatusEffects;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.item.ItemStack;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.network.ServerPlayerEntity;
import net.minecraft.util.math.BlockPos;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

public class PlayerSubmersionTracker {
    private static final Map<UUID, Integer> slurpTicks  = new HashMap<>();
    private static final Map<UUID, Integer> waterTicks  = new HashMap<>();
    private static final int THRESHOLD       = 30;  // 1.5 seconds
    private static final int SLURPY_DURATION = 200; // 10 seconds
    private static final int WITHER_DURATION = 100; // 5 seconds

    public static void registerEvents() {
        ServerTickEvents.END_SERVER_TICK.register(PlayerSubmersionTracker::onServerTick);
    }

    private static void onServerTick(MinecraftServer server) {
        for (PlayerEntity player : server.getPlayerManager().getPlayerList()) {
            UUID uuid = player.getUuid();
            boolean hasShield = isHoldingSlurpShield(player);

            // Slurp shield cures Slurpy effect AND clears all locked hearts (unless world is cursed)
            if (hasShield && !CurseTracker.isWorldCursed() && player instanceof ServerPlayerEntity sp) {
                if (sp.hasStatusEffect(NoahArkMod.SLURPY)) sp.removeStatusEffect(NoahArkMod.SLURPY);
                if (LockedHeartTracker.getLocked(sp) > 0) LockedHeartTracker.clearLocked(sp);
            }

            if (isInSlurp(player)) {
                slurpTicks.merge(uuid, 1, Integer::sum);
                waterTicks.remove(uuid);
                int ticks = slurpTicks.getOrDefault(uuid, 0);
                if (hasShield) {
                    // Shield drains 1 durability per 10 ticks in slurp
                    if (ticks % 10 == 0) drainShield(player, 1);
                } else {
                    if (ticks == THRESHOLD) {
                        player.addStatusEffect(new StatusEffectInstance(NoahArkMod.SLURPY, SLURPY_DURATION, 0, false, true));
                    }
                    // Lock 1 heart per 60 ticks of slurp exposure
                    if (ticks > 0 && ticks % 60 == 0 && player instanceof ServerPlayerEntity sp) {
                        LockedHeartTracker.addLocked(sp, 1);
                    }
                }
            } else if (player.isSubmergedInWater()) {
                waterTicks.merge(uuid, 1, Integer::sum);
                slurpTicks.remove(uuid);
                if (waterTicks.get(uuid) == THRESHOLD) {
                    player.addStatusEffect(new StatusEffectInstance(StatusEffects.WITHER, WITHER_DURATION, 0, false, false));
                }
            } else {
                slurpTicks.remove(uuid);
                waterTicks.remove(uuid);
            }
        }
    }

    private static boolean isInSlurp(PlayerEntity player) {
        BlockPos eyePos = BlockPos.ofFloored(player.getX(), player.getEyeY(), player.getZ());
        var fluid = player.getWorld().getFluidState(eyePos).getFluid();
        return fluid == NoahArkMod.STILL_SLURP || fluid == NoahArkMod.FLOWING_SLURP;
    }

    public static void drainShield(PlayerEntity player, int amount) {
        if (!(player instanceof ServerPlayerEntity sp)) return;
        ItemStack main = player.getMainHandStack();
        if (main.isOf(NoahArkMod.SLURP_SHIELD)) {
            main.damage(amount, sp, EquipmentSlot.MAINHAND);
        } else {
            ItemStack off = player.getOffHandStack();
            if (off.isOf(NoahArkMod.SLURP_SHIELD)) {
                off.damage(amount, sp, EquipmentSlot.OFFHAND);
            }
        }
    }

    private static boolean isHoldingSlurpShield(PlayerEntity player) {
        ItemStack main = player.getMainHandStack();
        ItemStack off  = player.getOffHandStack();
        return main.isOf(NoahArkMod.SLURP_SHIELD) || off.isOf(NoahArkMod.SLURP_SHIELD);
    }

    public static void reset() {
        slurpTicks.clear();
        waterTicks.clear();
    }
}
