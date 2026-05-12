package com.noahsark.world;

import com.noahsark.NoahArkMod;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.block.PillarBlock;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;

import java.util.ArrayList;
import java.util.List;

public final class NoahsArkPlacer {
    private static final int LENGTH     = 20;
    private static final int HALF_WIDTH = 3;
    private static final int MAST_Z     = 5;

    // Drift dirs for wobble: N, E, S, W
    private static final int[][] DRIFT = {{0,-1},{1,0},{0,1},{-1,0}};
    // Drift every 8 minutes of flood time (relative ticks, not world ticks)
    private static final int DRIFT_INTERVAL = 9600;

    private NoahsArkPlacer() {}

    /** Places the ark as real world blocks and registers the placement. */
    public static void place(ServerWorld world, BlockPos origin, PlayerEntity player) {
        Direction facing = player.getHorizontalFacing();
        placeStructure(world, origin, facing);
        NoahArkMod.placedArks.add(new NoahArkMod.ArkPlacement(origin, facing));
    }

    /**
     * Called every flood-phase tick from StormManager.
     * Rises arks when flood reaches them, and drifts them occasionally.
     */
    public static void tickArks(ServerWorld world, int floodY, int floodTick) {
        for (var ap : NoahArkMod.placedArks) {
            // Rise: flood above current keel → shift up 1 block
            if (floodY > ap.origin.getY()) {
                shiftArk(world, ap, 0, 1, 0);
            }

            // Wobble: only once flood has reached this ark
            if (floodY >= ap.origin.getY()) {
                ap.driftTicks++;
                if (ap.driftTicks >= DRIFT_INTERVAL) {
                    ap.driftTicks = 0;
                    ap.driftDir = world.getRandom().nextInt(4);
                    int[] d = DRIFT[ap.driftDir];
                    shiftArk(world, ap, d[0], 0, d[1]);
                }
            }
        }
    }

    /** Remove all ark blocks at current position, place them at (origin + dx, dy, dz), update origin. */
    private static void shiftArk(ServerWorld world, NoahArkMod.ArkPlacement ap, int dx, int dy, int dz) {
        List<BlockPos> oldPositions = allPositions(ap.origin, ap.facing);
        List<BlockState> states = new ArrayList<>();
        for (BlockPos pos : oldPositions) {
            states.add(world.getBlockState(pos));
            world.removeBlock(pos, false);
        }
        BlockPos newOrigin = ap.origin.add(dx, dy, dz);
        for (int i = 0; i < oldPositions.size(); i++) {
            BlockPos oldPos = oldPositions.get(i);
            BlockPos newPos = new BlockPos(oldPos.getX() + dx, oldPos.getY() + dy, oldPos.getZ() + dz);
            world.setBlockState(newPos, states.get(i), 3);
        }
        ap.origin = newOrigin;
    }

    /** All block positions for an ark at the given origin + facing. */
    private static List<BlockPos> allPositions(BlockPos origin, Direction facing) {
        Direction.Axis shipAxis = facing.getAxis();
        List<BlockPos> out = new ArrayList<>();

        // Mirror the place() loop structure, collecting positions
        for (int lz = 0; lz < LENGTH; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) out.add(xform(origin, lx, 0, lz, facing));
        }
        for (int lz = 0; lz < LENGTH; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) out.add(xform(origin, lx, 1, lz, facing));
        }
        for (int lz = 4; lz <= 15; lz++) {
            boolean pillar = (lz == 4 || lz == 9 || lz == 14);
            out.add(xform(origin, -3, 2, lz, facing));
            out.add(xform(origin,  3, 2, lz, facing));
            if (pillar) {
                out.add(xform(origin, -3, 3, lz, facing));
                out.add(xform(origin,  3, 3, lz, facing));
            }
        }
        for (int lx = -3; lx <= 3; lx++) {
            if (lx != 0) {
                out.add(xform(origin, lx, 2, 3, facing));
                out.add(xform(origin, lx, 3, 3, facing));
            }
            out.add(xform(origin, lx, 2, 16, facing));
            out.add(xform(origin, lx, 3, 16, facing));
        }
        for (int lz = 0; lz <= 2; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) {
                out.add(xform(origin, lx, 2, lz, facing));
                out.add(xform(origin, lx, 3, lz, facing));
                out.add(xform(origin, lx, 2, LENGTH-1-lz, facing));
                out.add(xform(origin, lx, 3, LENGTH-1-lz, facing));
            }
        }
        for (int lz = 4; lz <= 15; lz++) {
            if (lz == 4 || lz == 9 || lz == 14) continue;
            out.add(xform(origin, -2, 2, lz, facing));
            out.add(xform(origin,  2, 2, lz, facing));
        }
        for (int lz = 0; lz <= 6; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) out.add(xform(origin, lx, 4, lz, facing));
        }
        for (int lz = 13; lz < LENGTH; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) out.add(xform(origin, lx, 4, lz, facing));
        }
        for (int lz = 7; lz <= 12; lz++) {
            out.add(xform(origin, -3, 4, lz, facing));
            out.add(xform(origin,  3, 4, lz, facing));
        }
        for (int tz : new int[]{6, 11}) {
            out.add(xform(origin, -2, 2, tz, facing));
            out.add(xform(origin,  2, 2, tz, facing));
        }
        for (int my = 5; my <= 9; my++) out.add(xform(origin, 0, my, MAST_Z, facing));
        for (int sy = 6; sy <= 9; sy++) {
            for (int sx = -2; sx <= 2; sx++) {
                if (sx != 0) out.add(xform(origin, sx, sy, MAST_Z, facing));
            }
        }
        for (int sx = -2; sx <= 2; sx++) out.add(xform(origin, sx, 9, MAST_Z, facing));

        return out;
    }

    private static void placeStructure(ServerWorld world, BlockPos origin, Direction facing) {
        Direction.Axis shipAxis = facing.getAxis();
        BlockState hullLog = NoahArkMod.GOPHER_LOG.getDefaultState().with(PillarBlock.AXIS, shipAxis);
        BlockState mastLog = NoahArkMod.GOPHER_LOG.getDefaultState().with(PillarBlock.AXIS, Direction.Axis.Y);
        BlockState planks  = Blocks.OAK_PLANKS.getDefaultState();
        BlockState glass   = Blocks.GLASS.getDefaultState();
        BlockState wool    = Blocks.WHITE_WOOL.getDefaultState();
        BlockState shield  = NoahArkMod.SLURP_SHIELD_BLOCK.getDefaultState();
        BlockState torch   = Blocks.TORCH.getDefaultState();

        for (int lz = 0; lz < LENGTH; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) set(world, origin, lx, 0, lz, facing, hullLog);
        }
        for (int lz = 0; lz < LENGTH; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) {
                boolean edge = Math.abs(lx) == hw || lz == 0 || lz == LENGTH - 1;
                set(world, origin, lx, 1, lz, facing, edge ? shield : planks);
            }
        }
        for (int lz = 4; lz <= 15; lz++) {
            boolean pillar = (lz == 4 || lz == 9 || lz == 14);
            set(world, origin, -3, 2, lz, facing, pillar ? planks : glass);
            set(world, origin,  3, 2, lz, facing, pillar ? planks : glass);
            if (pillar) {
                set(world, origin, -3, 3, lz, facing, planks);
                set(world, origin,  3, 3, lz, facing, planks);
            }
        }
        for (int lx = -3; lx <= 3; lx++) {
            if (lx != 0) {
                set(world, origin, lx, 2, 3, facing, planks);
                set(world, origin, lx, 3, 3, facing, planks);
            }
            set(world, origin, lx, 2, 16, facing, planks);
            set(world, origin, lx, 3, 16, facing, planks);
        }
        for (int lz = 0; lz <= 2; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) {
                set(world, origin, lx, 2, lz,          facing, planks);
                set(world, origin, lx, 3, lz,          facing, planks);
                set(world, origin, lx, 2, LENGTH-1-lz, facing, planks);
                set(world, origin, lx, 3, LENGTH-1-lz, facing, planks);
            }
        }
        for (int lz = 4; lz <= 15; lz++) {
            if (lz == 4 || lz == 9 || lz == 14) continue;
            set(world, origin, -2, 2, lz, facing, shield);
            set(world, origin,  2, 2, lz, facing, shield);
        }
        for (int lz = 0; lz <= 6; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) set(world, origin, lx, 4, lz, facing, planks);
        }
        for (int lz = 13; lz < LENGTH; lz++) {
            int hw = hw(lz);
            for (int lx = -hw; lx <= hw; lx++) set(world, origin, lx, 4, lz, facing, planks);
        }
        for (int lz = 7; lz <= 12; lz++) {
            set(world, origin, -3, 4, lz, facing, planks);
            set(world, origin,  3, 4, lz, facing, planks);
        }
        for (int tz : new int[]{6, 11}) {
            set(world, origin, -2, 2, tz, facing, torch);
            set(world, origin,  2, 2, tz, facing, torch);
        }
        for (int my = 5; my <= 9; my++) set(world, origin, 0, my, MAST_Z, facing, mastLog);
        for (int sy = 6; sy <= 9; sy++) {
            for (int sx = -2; sx <= 2; sx++) {
                if (sx != 0) set(world, origin, sx, sy, MAST_Z, facing, wool);
            }
        }
        for (int sx = -2; sx <= 2; sx++) {
            set(world, origin, sx, 9, MAST_Z, facing, sx == 0 ? mastLog : wool);
        }
    }

    private static int hw(int lz) {
        return Math.min(Math.min(lz, LENGTH - 1 - lz), HALF_WIDTH);
    }

    private static void set(ServerWorld world, BlockPos o, int lx, int ly, int lz, Direction facing, BlockState state) {
        world.setBlockState(xform(o, lx, ly, lz, facing), state, 3);
    }

    private static BlockPos xform(BlockPos o, int lx, int ly, int lz, Direction facing) {
        return switch (facing) {
            case SOUTH -> o.add( lx, ly,  lz);
            case NORTH -> o.add(-lx, ly, -lz);
            case EAST  -> o.add( lz, ly, -lx);
            case WEST  -> o.add(-lz, ly,  lx);
            default    -> o.add( lx, ly,  lz);
        };
    }
}
