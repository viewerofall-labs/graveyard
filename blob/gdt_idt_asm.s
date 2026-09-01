.global gdt_load
.global idt_load
.global isr0
.global isr14
.extern exception_handler

# Reload segment registers with GDT code (0x08) and data (0x10)
gdt_load:
    lgdt (%rdi)
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss
    # Long jump to reload CS register
    pushq $0x08
    leaq .reload_cs(%rip), %rax
    pushq %rax
    lretq
.reload_cs:
    ret

# Load IDT Register
idt_load:
    lidt (%rdi)
    sti                 # Enable interrupts
    ret

# ISR 0: Divide-by-Zero (No error code pushed by CPU)
isr0:
    pushq $0            # Push dummy error code
    pushq $0            # Push vector number 0
    jmp isr_common

# ISR 14: Page Fault (Error code automatically pushed by CPU)
isr14:
    pushq $14           # Push vector number 14
    jmp isr_common

isr_common:
    push %rax
    push %rcx
    push %rdx
    push %rsi
    push %rdi
    push %r8
    push %r9
    push %r10
    push %r11

    mov 72(%rsp), %rdi  # Argument 1: Vector
    mov 80(%rsp), %rsi  # Argument 2: Error Code
    lea 88(%rsp), %rdx  # Argument 3: Pointer to Interrupt Frame

    call exception_handler

    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rdi
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rax
    add $16, %rsp       # Clean vector & error code
    iretq
