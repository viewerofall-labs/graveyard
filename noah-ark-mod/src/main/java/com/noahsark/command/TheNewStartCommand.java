package com.noahsark.command;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.noahsark.NoahArkMod;
import com.noahsark.event.StormManager;
import net.minecraft.server.command.CommandManager;
import net.minecraft.server.command.ServerCommandSource;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.text.Text;
import net.minecraft.world.World;

public class TheNewStartCommand {
    public static void register(CommandDispatcher<ServerCommandSource> dispatcher) {
        dispatcher.register(CommandManager.literal("thenewstart")
            .requires(src -> src.hasPermissionLevel(2))
            .then(CommandManager.argument("phase", IntegerArgumentType.integer(1, 3))
                .executes(ctx -> {
                    int phase = IntegerArgumentType.getInteger(ctx, "phase");
                    ServerCommandSource src = ctx.getSource();
                    ServerWorld world = src.getServer().getWorld(World.OVERWORLD);
                    return switch (phase) {
                        case 1 -> startStorm(src);
                        case 2 -> skipToFlood(src);
                        case 3 -> cleanFlood(src, world);
                        default -> 0;
                    };
                })
            )
        );
    }

    private static int startStorm(ServerCommandSource src) {
        if (NoahArkMod.stormManager != null && NoahArkMod.stormManager.isActive()) {
            src.sendError(Text.literal("Storm already active."));
            return 0;
        }
        NoahArkMod.stormManager = new StormManager();
        src.sendFeedback(() -> Text.literal("§8[§5Noah's Ark§8] §rThe storm begins."), true);
        return 1;
    }

    private static int skipToFlood(ServerCommandSource src) {
        if (NoahArkMod.stormManager == null) {
            NoahArkMod.stormManager = new StormManager();
        }
        NoahArkMod.stormManager.skipToFlood();
        src.sendFeedback(() -> Text.literal("§4§lTHE FLOOD IS HERE."), true);
        return 1;
    }

    private static int cleanFlood(ServerCommandSource src, ServerWorld world) {
        if (world == null) {
            src.sendError(Text.literal("Overworld not loaded."));
            return 0;
        }
        int removed = StormManager.cleanFlood(world);
        if (NoahArkMod.stormManager != null) {
            NoahArkMod.stormManager.forceEnd();
        }
        src.sendFeedback(() -> Text.literal("§8[§5Noah's Ark§8] §rFlood cleared (" + removed + " blocks removed)."), false);
        return 1;
    }
}
