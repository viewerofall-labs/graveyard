#include "keyboard.h"
#include "io.h"

static const char scancode_ascii[128] = {
    0,  27, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\b',
  '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
     0, 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',   0,
   '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',   0, '*',   0,
   ' '
};

char keyboard_getchar(void) {
    // Poll PS/2 Status Register (port 0x64) until output buffer is full (bit 0 set)
    while ((inb(0x64) & 1) == 0);

    uint8_t scancode = inb(0x60);
    // Ignore key release events (bit 7 set)
    if (scancode & 0x80) {
        return 0;
    }

    return scancode_ascii[scancode];
}
