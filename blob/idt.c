#include "idt.h"
#include <stddef.h>

// Forward declarations from text rendering engine
void kprint(const char *str, uint32_t fg, uint32_t bg);

static struct idt_entry idt[256];
static struct idt_ptr   idtr;

extern void idt_load(struct idt_ptr *ptr);
extern void isr0(void); // Vector 0: Divide-by-Zero
extern void isr14(void); // Vector 14: Page Fault

void set_idt_gate(uint8_t num, uint64_t base, uint16_t sel, uint8_t flags) {
    idt[num].offset_low      = (base & 0xFFFF);
    idt[num].selector        = sel;
    idt[num].ist             = 0;
    idt[num].type_attributes = flags;
    idt[num].offset_mid      = (base >> 16) & 0xFFFF;
    idt[num].offset_high     = (base >> 32) & 0xFFFFFFFF;
    idt[num].zero            = 0;
}

void idt_init(void) {
    idtr.limit = (sizeof(struct idt_entry) * 256) - 1;
    idtr.base  = (uint64_t)&idt;

    // 0x8E = Present, Ring 0, 64-bit Interrupt Gate
    set_idt_gate(0, (uint64_t)isr0, 0x08, 0x8E);
    set_idt_gate(14, (uint64_t)isr14, 0x08, 0x8E);

    idt_load(&idtr);
}

// C Interrupt Handler called by ASM stubs
void exception_handler(uint64_t vec, uint64_t error_code, struct interrupt_frame *frame) {
    (void)error_code;
    (void)frame;

    if (vec == 0) {
        kprint("\n[CPU EXCEPTION] #DE: Divide-by-Zero Fault Detected!\n", 0x00FF0000, 0x001A1C23);
    } else if (vec == 14) {
        kprint("\n[CPU EXCEPTION] #PF: Page Fault Detected!\n", 0x00FF0000, 0x001A1C23);
    } else {
        kprint("\n[CPU EXCEPTION] Unknown Exception!\n", 0x00FF0000, 0x001A1C23);
    }

    // Halt CPU cleanly instead of rebooting/triple faulting
    while (1) { __asm__ volatile ("hlt"); }
}
