# Auto-generated Makefile from build.toml
# DO NOT EDIT MANUALLY
# Regenerate with: build-tool transpile

CARGO ?= cargo
GCC ?= gcc
PYINSTALLER ?= pyinstaller
CARGO ?= cargo
ZIG ?= zig
NASM ?= nasm
LD ?= ld

.PHONY: all clean help cipher gpu_glitch_x11 ok tpmenc picotemp maketoml gstress hi cstress lat bagel pitime shacmp

cipher:	cipher/Cargo.toml
	$(CARGO) build --release

exec/cipher:	cipher/Cargo.toml
	$(CARGO) build --release

gpu_glitch_x11:	gpu_glitch_x11.c
	$(GCC) -O2 -Wall -lX11 -lGLEW -lGL gpu_glitch_x11.c -c -o exec/gpu_glitch_x11

exec/gpu_glitch_x11:	gpu_glitch_x11.c
	$(GCC) -O2 -Wall -lX11 -lGLEW -lGL gpu_glitch_x11.c -c -o exec/gpu_glitch_x11

ok:	ok/Cargo.toml
	$(CARGO) build --release

exec/ok:	ok/Cargo.toml
	$(CARGO) build --release

tpmenc:	tpmenc/Cargo.toml
	$(CARGO) build --release

exec/tpmenc:	tpmenc/Cargo.toml
	$(CARGO) build --release

picotemp:	picotemp/Cargo.toml
	cd $(dirname picotemp/Cargo.toml) && $(CARGO) build --release --target thumbv6m-none-eabi && elf2uf2-rs target/thumbv6m-none-eabi/release/$(cargo metadata --format-version 1 --manifest-path picotemp/Cargo.toml | grep '"name"' | head -1 | cut -d'"' -f4 | tr '-' '_') -o pi/picotemp.uf2

pi/picotemp.uf2:	picotemp/Cargo.toml
	cd $(dirname picotemp/Cargo.toml) && $(CARGO) build --release --target thumbv6m-none-eabi && elf2uf2-rs target/thumbv6m-none-eabi/release/$(cargo metadata --format-version 1 --manifest-path picotemp/Cargo.toml | grep '"name"' | head -1 | cut -d'"' -f4 | tr '-' '_') -o pi/picotemp.uf2

maketoml:	maketoml/Cargo.toml
	$(CARGO) build --release --quiet

exec/tomk:	maketoml/Cargo.toml
	$(CARGO) build --release --quiet

gstress:	gstress/Cargo.toml
	$(CARGO) build --release

exec/gstress:	gstress/Cargo.toml
	$(CARGO) build --release

hi:	hi/Cargo.toml
	$(CARGO) build --release

exec/hi:	hi/Cargo.toml
	$(CARGO) build --release

cstress:	cstress/Cargo.toml
	$(CARGO) build --release

exec/cstress:	cstress/Cargo.toml
	$(CARGO) build --release

lat:	lat.c
	$(GCC) -O2 -Wall lat.c -c -o exec/lat

exec/lat:	lat.c
	$(GCC) -O2 -Wall lat.c -c -o exec/lat

bagel:	bagel/Cargo.toml
	$(CARGO) build --release

exec/bagel:	bagel/Cargo.toml
	$(CARGO) build --release

pitime:	pitime/Cargo.toml
	cd $(dirname pitime/Cargo.toml) && $(CARGO) build --release --target thumbv6m-none-eabi && elf2uf2-rs target/thumbv6m-none-eabi/release/$(cargo metadata --format-version 1 --manifest-path pitime/Cargo.toml | grep '"name"' | head -1 | cut -d'"' -f4 | tr '-' '_') -o pi/pitime.uf2

pi/pitime.uf2:	pitime/Cargo.toml
	cd $(dirname pitime/Cargo.toml) && $(CARGO) build --release --target thumbv6m-none-eabi && elf2uf2-rs target/thumbv6m-none-eabi/release/$(cargo metadata --format-version 1 --manifest-path pitime/Cargo.toml | grep '"name"' | head -1 | cut -d'"' -f4 | tr '-' '_') -o pi/pitime.uf2

shacmp:	shacmp/Cargo.toml
	$(CARGO) build --release

exec/shacmp:	shacmp/Cargo.toml
	$(CARGO) build --release

all: cipher gpu_glitch_x11 ok tpmenc picotemp maketoml gstress hi cstress lat bagel pitime shacmp

clean:
	@rm -f exec/cipher
	@rm -f exec/gpu_glitch_x11
	@rm -f exec/ok
	@rm -f exec/tpmenc
	@rm -f pi/picotemp.uf2
	@rm -f exec/tomk
	@rm -f exec/gstress
	@rm -f exec/hi
	@rm -f exec/cstress
	@rm -f exec/lat
	@rm -f exec/bagel
	@rm -f pi/pitime.uf2
	@rm -f exec/shacmp
	@cargo clean 2>/dev/null || true

help:
	@echo "Available targets:"
	@echo "  - cipher"
	@echo "  - gpu_glitch_x11"
	@echo "  - ok"
	@echo "  - tpmenc"
	@echo "  - picotemp"
	@echo "  - maketoml"
	@echo "  - gstress"
	@echo "  - hi"
	@echo "  - cstress"
	@echo "  - lat"
	@echo "  - bagel"
	@echo "  - pitime"
	@echo "  - shacmp"
