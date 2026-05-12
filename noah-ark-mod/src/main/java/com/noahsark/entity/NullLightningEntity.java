package com.noahsark.entity;

import com.noahsark.NoahArkMod;
import net.minecraft.entity.EntityType;
import net.minecraft.entity.LightningEntity;
import net.minecraft.entity.SpawnReason;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.world.World;

public class NullLightningEntity extends LightningEntity {
    public NullLightningEntity(EntityType<? extends LightningEntity> type, World world) {
        super(type, world);
    }

    @Override
    public void tick() {
        super.tick();
        if (this.getWorld().isClient() || this.age != 1) return;
        if (this.getWorld().getRandom().nextFloat() < 0.10f) {
            var world = (ServerWorld) this.getWorld();
            SlurplingEntity mob = NoahArkMod.SLURPLING.create(world, SpawnReason.EVENT);
            if (mob != null) {
                mob.setPosition(this.getX(), this.getY(), this.getZ());
                world.spawnEntity(mob);
            }
        }
    }
}
