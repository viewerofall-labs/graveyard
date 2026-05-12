package com.noahsark.fluid;

import com.noahsark.NoahArkMod;
import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.FluidBlock;
import net.minecraft.block.entity.BlockEntity;
import net.minecraft.fluid.FlowableFluid;
import net.minecraft.fluid.Fluid;
import net.minecraft.fluid.FluidState;
import net.minecraft.item.Item;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.state.StateManager;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import net.minecraft.world.BlockView;
import net.minecraft.world.WorldAccess;
import net.minecraft.world.WorldView;

public abstract class SlurpFluid extends FlowableFluid {

    @Override
    public Fluid getStill() {
        return NoahArkMod.STILL_SLURP;
    }

    @Override
    public Fluid getFlowing() {
        return NoahArkMod.FLOWING_SLURP;
    }

    @Override
    public Item getBucketItem() {
        return NoahArkMod.SLURP_BUCKET;
    }

    @Override
    public BlockState toBlockState(FluidState state) {
        return NoahArkMod.SLURP_BLOCK.getDefaultState().with(FluidBlock.LEVEL, getBlockStateLevel(state));
    }

    @Override
    public boolean matchesType(Fluid fluid) {
        return fluid == NoahArkMod.STILL_SLURP || fluid == NoahArkMod.FLOWING_SLURP;
    }

    @Override
    protected boolean isInfinite(ServerWorld world) {
        return false;
    }

    @Override
    protected void beforeBreakingBlock(WorldAccess world, BlockPos pos, BlockState state) {
        final BlockEntity be = state.hasBlockEntity() ? world.getBlockEntity(pos) : null;
        Block.dropStacks(state, world, pos, be);
    }

    // Lava-like: max distance 2, level decrease 2, tick rate 30
    @Override
    public int getMaxFlowDistance(WorldView world) {
        return 2;
    }

    @Override
    public int getLevelDecreasePerBlock(WorldView world) {
        return 2;
    }

    @Override
    public int getTickRate(WorldView world) {
        return 30;
    }

    @Override
    protected float getBlastResistance() {
        return 100.0f;
    }

    @Override
    public boolean canBeReplacedWith(FluidState state, BlockView world, BlockPos pos, Fluid fluid, Direction direction) {
        return false;
    }

    public static class Still extends SlurpFluid {
        @Override
        public boolean isStill(FluidState state) {
            return true;
        }

        @Override
        public int getLevel(FluidState state) {
            return 8;
        }
    }

    public static class Flowing extends SlurpFluid {
        @Override
        protected void appendProperties(StateManager.Builder<Fluid, FluidState> builder) {
            super.appendProperties(builder);
            builder.add(LEVEL);
        }

        @Override
        public boolean isStill(FluidState state) {
            return false;
        }

        @Override
        public int getLevel(FluidState state) {
            return state.get(LEVEL);
        }
    }
}
