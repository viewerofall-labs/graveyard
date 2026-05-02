# Auto-generated Makefile from build.toml
# DO NOT EDIT MANUALLY
# Regenerate with: build-tool transpile

GCC ?= gcc
CARGO ?= cargo
PYINSTALLER ?= pyinstaller
LD ?= ld
ZIG ?= zig
NASM ?= nasm

KERNEL_FLAGS = -ffreestanding -fno-builtin
LINK_SCRIPT = linker.ld

.PHONY: all clean help bootloader build_tool kernel_elf kernel libc rust_runtime

bootloader:	asm/boot.s
	$(NASM) -f elf64 asm/boot.s -o boot.o

boot.o:	asm/boot.s
	$(NASM) -f elf64 asm/boot.s -o boot.o

build_tool:	python/build_tool.py
	$(PYINSTALLER) --onefile python/build_tool.py -o build_tool

kernel_elf:	unknown bootloader kernel libc rust_runtime
	$(LD) -T linker.ld unknown -o kernel.elf

kernel.elf:	unknown bootloader kernel libc rust_runtime
	$(LD) -T linker.ld unknown -o kernel.elf

kernel:	c/kernel.c boot.o
	$(GCC) -Wall -O2 -ffreestanding -fno-builtin c/kernel.c -c -o kernel.o

kernel.o:	c/kernel.c boot.o
	$(GCC) -Wall -O2 -ffreestanding -fno-builtin c/kernel.c -c -o kernel.o

libc:	c/libc.c
	$(GCC) -Wall -O2 -fPIC c/libc.c -c -o libc.o

libc.o:	c/libc.c
	$(GCC) -Wall -O2 -fPIC c/libc.c -c -o libc.o

rust_runtime:	rust/Cargo.toml
	$(CARGO) build --release

target/release/libruntime.a:	rust/Cargo.toml
	$(CARGO) build --release

all: bootloader build_tool kernel_elf kernel libc rust_runtime

clean:
	@rm -f boot.o
	@rm -f build_tool
	@rm -f kernel.elf
	@rm -f kernel.o
	@rm -f libc.o
	@rm -f target/release/libruntime.a
	@cargo clean 2>/dev/null || true

help:
	@echo "Available targets:"
	@echo "  - bootloader"
	@echo "  - build_tool"
	@echo "  - kernel_elf"
	@echo "  - kernel"
	@echo "  - libc"
	@echo "  - rust_runtime"
