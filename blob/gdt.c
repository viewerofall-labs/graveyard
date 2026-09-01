#include "gdt.h"

static struct gdt_entry gdt[3];
static struct gdt_ptr   gdtr;

// Assembly routine to reload segment registers
extern void gdt_load(struct gdt_ptr *ptr);

static void set_gdt_entry(int num, uint32_t base, uint32_t limit, uint8_t access, uint8_t gran) {
    gdt[num].base_low    = (base & 0xFFFF);
    gdt[num].base_middle = (base >> 16) & 0xFF;
    gdt[num].base_high   = (base >> 24) & 0xFF;
    gdt[num].limit_low   = (limit & 0xFFFF);
    gdt[num].granularity = (limit >> 16) & 0x0F;
    gdt[num].granularity |= gran & 0xF0;
    gdt[num].access      = access;
}

void gdt_init(void) {
    gdtr.limit = (sizeof(struct gdt_entry) * 3) - 1;
    gdtr.base  = (uint64_t)&gdt;

    // 0x00: Null Descriptor
    set_gdt_entry(0, 0, 0, 0, 0);

    // 0x08: Kernel 64-bit Code Segment (Executable, Readable, Long Mode)
    set_gdt_entry(1, 0, 0xFFFFFFFF, 0x9A, 0xAF);

    // 0x10: Kernel 64-bit Data Segment (Writable)
    set_gdt_entry(2, 0, 0xFFFFFFFF, 0x92, 0xCF);

    gdt_load(&gdtr);
}
