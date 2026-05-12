package com.noahsark;

import com.noahsark.block.GopherLogBlock;
import com.noahsark.block.SlurpBlock;
import com.noahsark.block.SlurpShieldBlock;
import com.noahsark.command.TheNewStartCommand;
import com.noahsark.effect.SlurpyEffect;
import com.noahsark.effect.WorldCursedEffect;
import com.noahsark.entity.NullLightningEntity;
import com.noahsark.entity.SlurplingEntity;
import com.noahsark.event.CurseTracker;
import com.noahsark.event.PlayerSubmersionTracker;
import com.noahsark.event.StormManager;
import com.noahsark.event.WelcomeHandler;
import com.noahsark.fluid.SlurpFluid;
import com.noahsark.item.ArkItem;
import com.noahsark.item.SlurpShieldItem;
import com.noahsark.world.GopherTreeFeature;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.biome.v1.BiomeModifications;
import net.fabricmc.fabric.api.biome.v1.BiomeSelectors;
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback;
import net.fabricmc.fabric.api.itemgroup.v1.FabricItemGroup;
import net.fabricmc.fabric.api.entity.event.v1.ServerPlayerEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.fabricmc.fabric.api.networking.v1.ServerPlayConnectionEvents;
import net.fabricmc.fabric.api.object.builder.v1.entity.FabricDefaultAttributeRegistry;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.SpawnGroup;
import net.minecraft.block.AbstractBlock;
import net.minecraft.block.Block;
import net.minecraft.block.Blocks;
import net.minecraft.entity.effect.StatusEffect;
import net.minecraft.fluid.FlowableFluid;
import net.minecraft.item.BlockItem;
import net.minecraft.item.BucketItem;
import net.minecraft.item.Item;
import net.minecraft.item.ItemGroup;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import net.minecraft.item.SpawnEggItem;
import net.minecraft.registry.Registries;
import net.minecraft.registry.Registry;
import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import net.minecraft.world.World;
import net.minecraft.world.gen.GenerationStep;
import net.minecraft.world.gen.feature.DefaultFeatureConfig;
import net.minecraft.world.gen.feature.Feature;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.List;

public class NoahArkMod implements ModInitializer {
    public static final String MOD_ID = "noahsark";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    // Registration order: fluids → blocks → items → features
    public static final FlowableFluid STILL_SLURP   = Registry.register(Registries.FLUID, Identifier.of(MOD_ID, "slurp"),         new SlurpFluid.Still());
    public static final FlowableFluid FLOWING_SLURP = Registry.register(Registries.FLUID, Identifier.of(MOD_ID, "flowing_slurp"), new SlurpFluid.Flowing());

    public static final Block SLURP_BLOCK = Registry.register(Registries.BLOCK, Identifier.of(MOD_ID, "slurp"),
        new SlurpBlock(STILL_SLURP, AbstractBlock.Settings.copy(Blocks.WATER)
            .registryKey(RegistryKey.of(RegistryKeys.BLOCK, Identifier.of(MOD_ID, "slurp")))));

    public static final Block GOPHER_LOG = Registry.register(Registries.BLOCK, Identifier.of(MOD_ID, "gopher_log"),
        new GopherLogBlock(AbstractBlock.Settings.copy(Blocks.OAK_LOG)
            .registryKey(RegistryKey.of(RegistryKeys.BLOCK, Identifier.of(MOD_ID, "gopher_log")))));

    public static final Block SLURP_SHIELD_BLOCK = Registry.register(Registries.BLOCK, Identifier.of(MOD_ID, "slurp_shield_block"),
        new SlurpShieldBlock(AbstractBlock.Settings.create().strength(3.5f, 6.0f).requiresTool()
            .registryKey(RegistryKey.of(RegistryKeys.BLOCK, Identifier.of(MOD_ID, "slurp_shield_block")))));

    public static final Item SLURP_BUCKET = Registry.register(Registries.ITEM, Identifier.of(MOD_ID, "slurp_bucket"),
        new BucketItem(STILL_SLURP, new Item.Settings().maxCount(1).recipeRemainder(Items.BUCKET)
            .registryKey(RegistryKey.of(RegistryKeys.ITEM, Identifier.of(MOD_ID, "slurp_bucket")))));

    public static final Item GOPHER_LOG_ITEM = Registry.register(Registries.ITEM, Identifier.of(MOD_ID, "gopher_log"),
        new BlockItem(GOPHER_LOG, new Item.Settings()
            .registryKey(RegistryKey.of(RegistryKeys.ITEM, Identifier.of(MOD_ID, "gopher_log")))));

    public static final Item SLURP_SHIELD = Registry.register(Registries.ITEM, Identifier.of(MOD_ID, "slurp_shield"),
        new SlurpShieldItem(new Item.Settings().maxCount(1).maxDamage(100)
            .registryKey(RegistryKey.of(RegistryKeys.ITEM, Identifier.of(MOD_ID, "slurp_shield")))));

    public static final Item SLURP_SHIELD_BLOCK_ITEM = Registry.register(Registries.ITEM, Identifier.of(MOD_ID, "slurp_shield_block"),
        new BlockItem(SLURP_SHIELD_BLOCK, new Item.Settings()
            .registryKey(RegistryKey.of(RegistryKeys.ITEM, Identifier.of(MOD_ID, "slurp_shield_block")))));

    public static final Item ARK = Registry.register(Registries.ITEM, Identifier.of(MOD_ID, "ark"),
        new ArkItem(new Item.Settings().maxCount(1)
            .registryKey(RegistryKey.of(RegistryKeys.ITEM, Identifier.of(MOD_ID, "ark")))));

    public static final Feature<DefaultFeatureConfig> GOPHER_TREE_FEATURE = Registry.register(
        Registries.FEATURE, Identifier.of(MOD_ID, "gopher_tree"),
        new GopherTreeFeature(DefaultFeatureConfig.CODEC));

    public static final RegistryKey<ItemGroup> ITEM_GROUP_KEY = RegistryKey.of(RegistryKeys.ITEM_GROUP, Identifier.of(MOD_ID, "main"));

    public static RegistryEntry.Reference<StatusEffect> SLURPY;
    public static RegistryEntry.Reference<StatusEffect> WORLD_CURSED;

    public static final EntityType<SlurplingEntity> SLURPLING = Registry.register(
        Registries.ENTITY_TYPE,
        Identifier.of(MOD_ID, "slurpling"),
        EntityType.Builder.create(SlurplingEntity::new, SpawnGroup.MONSTER)
            .dimensions(0.6f, 1.95f)
            .build(RegistryKey.of(RegistryKeys.ENTITY_TYPE, Identifier.of(MOD_ID, "slurpling")))
    );

    public static final EntityType<NullLightningEntity> NULL_LIGHTNING = Registry.register(
        Registries.ENTITY_TYPE,
        Identifier.of(MOD_ID, "null_lightning"),
        EntityType.Builder.<NullLightningEntity>create(NullLightningEntity::new, SpawnGroup.MISC)
            .dimensions(0f, 0f)
            .build(RegistryKey.of(RegistryKeys.ENTITY_TYPE, Identifier.of(MOD_ID, "null_lightning")))
    );

    @SuppressWarnings("unchecked")
    public static final Item SLURPLING_SPAWN_EGG = Registry.register(Registries.ITEM, Identifier.of(MOD_ID, "slurpling_spawn_egg"),
        new SpawnEggItem((net.minecraft.entity.EntityType<? extends net.minecraft.entity.mob.MobEntity>)(net.minecraft.entity.EntityType<?>)SLURPLING,
            new Item.Settings().registryKey(RegistryKey.of(RegistryKeys.ITEM, Identifier.of(MOD_ID, "slurpling_spawn_egg")))));

    /** Mutable placement record — tracks the ark as it rises and drifts. */
    public static class ArkPlacement {
        public BlockPos origin;          // current keel origin (rises over time)
        public final Direction facing;
        public int driftTicks = 0;
        public int driftDir   = 0;       // 0–3: N/E/S/W horizontal drift index

        public ArkPlacement(BlockPos origin, Direction facing) {
            this.origin = origin;
            this.facing = facing;
        }
    }
    public static final List<ArkPlacement> placedArks = new ArrayList<>();

    public static StormManager stormManager;
    private int idleTicks  = 0;
    private int globalTick = 0;

    @Override
    public void onInitialize() {
        LOGGER.info("Noah's Ark mod initializing...");

        SLURPY       = Registry.registerReference(Registries.STATUS_EFFECT, Identifier.of(MOD_ID, "slurpy"),       new SlurpyEffect());
        WORLD_CURSED = Registry.registerReference(Registries.STATUS_EFFECT, Identifier.of(MOD_ID, "world_cursed"), new WorldCursedEffect());

        FabricDefaultAttributeRegistry.register(SLURPLING, SlurplingEntity.createAttributes());

        Registry.register(Registries.ITEM_GROUP, ITEM_GROUP_KEY, FabricItemGroup.builder()
            .icon(() -> new ItemStack(ARK))
            .displayName(Text.translatable("itemGroup.noahsark.main"))
            .entries((ctx, entries) -> {
                entries.add(ARK);
                entries.add(SLURP_BUCKET);
                entries.add(SLURP_SHIELD);
                entries.add(SLURP_SHIELD_BLOCK_ITEM);
                entries.add(GOPHER_LOG_ITEM);
                entries.add(SLURPLING_SPAWN_EGG);
            })
            .build());

        BiomeModifications.addFeature(
            BiomeSelectors.foundInOverworld(),
            GenerationStep.Feature.VEGETAL_DECORATION,
            RegistryKey.of(RegistryKeys.PLACED_FEATURE, Identifier.of(MOD_ID, "gopher_tree")));

        CommandRegistrationCallback.EVENT.register((dispatcher, registryAccess, environment) ->
            TheNewStartCommand.register(dispatcher));

        PlayerSubmersionTracker.registerEvents();

        ServerTickEvents.END_SERVER_TICK.register(this::onServerTick);

        ServerPlayConnectionEvents.JOIN.register((handler, sender, server) -> {
            if (stormManager != null) stormManager.onPlayerJoin(handler.player);
            WelcomeHandler.onJoin(handler.player);
        });

        ServerPlayerEvents.AFTER_RESPAWN.register((oldPlayer, newPlayer, alive) -> {
            if (CurseTracker.isWorldCursed()) {
                CurseTracker.applyCurseEffect(newPlayer);
            }
        });
    }

    private void onServerTick(MinecraftServer server) {
        globalTick++;
        ServerWorld world = server.getWorld(World.OVERWORLD);
        if (world == null) return;

        if (stormManager != null && stormManager.isActive()) {
            stormManager.tick(world);
        } else {
            if (!world.getPlayers().isEmpty()) {
                idleTicks++;
                if (idleTicks % 12000 == 0 && world.getRandom().nextInt(5) == 0) {
                    stormManager = new StormManager();
                    idleTicks = 0;
                    for (var p : server.getPlayerManager().getPlayerList()) {
                        p.sendMessage(Text.literal("§8[§5Noah's Ark§8] §7The storm stirs on the horizon..."), false);
                    }
                }
            }
        }

        if (globalTick % 40 == 0) {
            CurseTracker.tick(world);
        }
    }
}
