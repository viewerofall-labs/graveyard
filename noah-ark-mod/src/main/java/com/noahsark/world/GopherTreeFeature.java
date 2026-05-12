package com.noahsark.world;

import com.mojang.serialization.Codec;
import com.noahsark.NoahArkMod;
import net.minecraft.block.PillarBlock;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import net.minecraft.world.gen.feature.DefaultFeatureConfig;
import net.minecraft.world.gen.feature.Feature;
import net.minecraft.world.gen.feature.util.FeatureContext;

public class GopherTreeFeature extends Feature<DefaultFeatureConfig> {
    public GopherTreeFeature(Codec<DefaultFeatureConfig> codec) {
        super(codec);
    }

    @Override
    public boolean generate(FeatureContext<DefaultFeatureConfig> ctx) {
        var world  = ctx.getWorld();
        var random = ctx.getRandom();
        var origin = ctx.getOrigin();

        if (world.getBlockState(origin.down()).isAir()) return false;

        int height = 6 + random.nextInt(3); // 6-8 blocks
        int cx = origin.getX(), cy = origin.getY(), cz = origin.getZ();

        var logState = NoahArkMod.GOPHER_LOG.getDefaultState()
                           .with(PillarBlock.AXIS, Direction.Axis.Y);

        for (int i = 0; i < height; i++) {
            var pos = new BlockPos(cx, cy + i, cz);
            if (!world.getBlockState(pos).isAir()) break;
            world.setBlockState(pos, logState, 3);

            // Zigzag: alternate shifting X and Z by ±1 each step
            if (i % 2 == 0) {
                cx += random.nextBoolean() ? 1 : -1;
            } else {
                cz += random.nextBoolean() ? 1 : -1;
            }
        }
        return true;
    }
}
