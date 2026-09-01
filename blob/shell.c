#include "shell.h"
#include "keyboard.h"
#include "io.h"
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// Forward declarations to kernel utilities
extern void kprint(const char *str, uint32_t fg_color, uint32_t bg_color);
extern void clear_screen(uint32_t color);
extern void draw_backspace(void); // Utility to erase character visually

#define MAX_BUFFER 256
#define MAX_ARGS 16
#define BG_COLOR 0x001A1C23

// String helper functions
static int strcmp(const char *s1, const char *s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++; s2++;
    }
    return *(const unsigned char *)s1 - *(const unsigned char *)s2;
}

static size_t strlen(const char *str) {
    size_t len = 0;
    while (str[len]) len++;
    return len;
}

static void print_prompt(void) {
    kprint("root@kernel", 0x0000FF00, BG_COLOR);
    kprint(":", 0x00FFFFFF, BG_COLOR);
    kprint("/# ", 0x0055FFFF, BG_COLOR);
}

// Built-in commands
static void cmd_help(void) {
    kprint("Available shell commands:\n", 0x00FFFF00, BG_COLOR);
    kprint("  help     - Show list of commands\n", 0x00FFFFFF, BG_COLOR);
    kprint("  clear    - Erase display contents\n", 0x00FFFFFF, BG_COLOR);
    kprint("  echo     - Print text to stdout\n", 0x00FFFFFF, BG_COLOR);
    kprint("  sysinfo  - Print kernel system information\n", 0x00FFFFFF, BG_COLOR);
    kprint("  reboot   - Soft restart machine\n", 0x00FFFFFF, BG_COLOR);
}

static void cmd_sysinfo(void) {
    kprint("OS Kernel: Bare-Metal x86_64\n", 0x0000FF00, BG_COLOR);
    kprint("Bootloader: Limine v7.x (HHDM Enabled)\n", 0x0000FF00, BG_COLOR);
    kprint("Arch: x86_64 Long Mode\n", 0x0000FF00, BG_COLOR);
}

static void cmd_reboot(void) {
    kprint("Rebooting machine...\n", 0x00FF0000, BG_COLOR);
    // Pulse 8042 keyboard controller reset line
    uint8_t good = 0x02;
    while (good & 0x02) {
        good = inb(0x64);
    }
    outb(0x64, 0xFE);
    while (1) { __asm__ volatile ("hlt"); }
}

// Execute Command Line
static void execute_command(char *line) {
    char *args[MAX_ARGS];
    int argc = 0;

    // Tokenize string space delimiters
    char *ptr = line;
    while (*ptr != '\0') {
        while (*ptr == ' ') *ptr++ = '\0';
        if (*ptr == '\0') break;
        if (argc < MAX_ARGS) {
            args[argc++] = ptr;
        }
        while (*ptr != '\0' && *ptr != ' ') ptr++;
    }

    if (argc == 0) return;

    if (strcmp(args[0], "help") == 0) {
        cmd_help();
    } else if (strcmp(args[0], "clear") == 0) {
        clear_screen(BG_COLOR);
    } else if (strcmp(args[0], "echo") == 0) {
        for (int i = 1; i < argc; i++) {
            kprint(args[i], 0x00FFFFFF, BG_COLOR);
            if (i < argc - 1) kprint(" ", 0x00FFFFFF, BG_COLOR);
        }
        kprint("\n", 0x00FFFFFF, BG_COLOR);
    } else if (strcmp(args[0], "sysinfo") == 0) {
        cmd_sysinfo();
    } else if (strcmp(args[0], "reboot") == 0) {
        cmd_reboot();
    } else {
        kprint("bash: ", 0x00FF0000, BG_COLOR);
        kprint(args[0], 0x00FF0000, BG_COLOR);
        kprint(": command not found\n", 0x00FF0000, BG_COLOR);
    }
}

void shell_init(void) {
    kprint("Interactive Bash Shell Ready.\n\n", 0x0000FF00, BG_COLOR);
    print_prompt();
}

void shell_run(void) {
    static char buffer[MAX_BUFFER];
    static size_t buf_idx = 0;

    char c = keyboard_getchar();
    if (c == 0) return;

    // Handle Backspace
    if (c == '\b') {
        if (buf_idx > 0) {
            buf_idx--;
            buffer[buf_idx] = '\0';
            draw_backspace();
        }
        return;
    }

    // Handle Enter / Execute
    if (c == '\n') {
        kprint("\n", 0x00FFFFFF, BG_COLOR);
        buffer[buf_idx] = '\0';
        execute_command(buffer);
        buf_idx = 0;
        print_prompt();
        return;
    }

    // Append standard character to input buffer
    if (buf_idx < MAX_BUFFER - 1) {
        buffer[buf_idx++] = c;
        buffer[buf_idx] = '\0';
        char str[2] = {c, '\0'};
        kprint(str, 0x00FFFFFF, BG_COLOR);
    }
}
