package com.noahsark.block;

import net.minecraft.block.AbstractBlock;
import net.minecraft.block.FluidBlock;
import net.minecraft.fluid.FlowableFluid;

public class SlurpBlock extends FluidBlock {
    public SlurpBlock(FlowableFluid fluid, AbstractBlock.Settings settings) {
        super(fluid, settings);
    }
}
