[BITS 32]
global _start
global keyboard_isr_stub
global load_idt
extern kernel_main
extern keyboard_handler_main

section .multiboot
align 4
    dd 0x1BADB002
    dd 0x00000003
    dd -(0x1BADB002 + 0x00000003)

section .text
_start:
    cli
    mov esp, stack_top
    call kernel_main

.loop:
    hlt
    jmp .loop

; Load IDT Pointer into CPU LIDT register
load_idt:
    mov eax, [esp + 4]
    lidt [eax]
    ret

; Low-level Assembly Stub for Keyboard IRQ1 (Vector 33)
keyboard_isr_stub:
    pusha                   ; Save all 32-bit registers (EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI)
    call keyboard_handler_main
    popa                    ; Restore registers
    iretd                   ; Return from interrupt (32-bit)

section .bss
align 16
stack_bottom:
    resb 16384
stack_top:
