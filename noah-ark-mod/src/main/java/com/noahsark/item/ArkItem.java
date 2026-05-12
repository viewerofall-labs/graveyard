package com.noahsark.item;

import com.noahsark.world.NoahsArkPlacer;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.item.Item;
import net.minecraft.item.ItemStack;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.ActionResult;
import net.minecraft.util.Hand;
import net.minecraft.util.hit.BlockHitResult;
import net.minecraft.util.hit.HitResult;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.World;

public class ArkItem extends Item {
    public ArkItem(Settings settings) {
        super(settings);
    }

    @Override
    public ActionResult use(World world, PlayerEntity user, Hand hand) {
        ItemStack stack = user.getStackInHand(hand);
        HitResult hit = user.raycast(10.0, 0.0f, false);
        if (!world.isClient && hit instanceof BlockHitResult blockHit) {
            BlockPos origin = blockHit.getBlockPos().up();
            NoahsArkPlacer.place((ServerWorld) world, origin, user);
            if (!user.getAbilities().creativeMode) stack.decrement(1);
            return ActionResult.SUCCESS;
        }
        return ActionResult.PASS;
    }
}
