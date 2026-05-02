# build-tool

Multi-language project builder. Define your build in TOML, transpile to Makefile, execute.

No GUI bloat. No custom formats. Just TOML → Makefile → Make.

## Installation

```bash
cargo build --release
sudo install -Dt /usr/local/bin target/release/build-tool
```

Or copy binary:
```bash
cp target/release/build-tool ~/.local/bin/
```

## Quick Start

1. Copy example config:
```bash
cp build.toml.example build.toml
```

2. Edit for your project:
```toml
[targets.my_program]
type = "c"
source = "src/main.c"
output = "my_program"
flags = "-Wall -O2"
```

3. Build:
```bash
build-tool build my_program
```

That's it. The tool transpiles `build.toml` → `Makefile` and runs `make my_program`.

## Commands

### `build-tool transpile [--output <file>]`
Generate Makefile from build.toml (default: Makefile).
```bash
build-tool transpile
build-tool transpile --output build.mk
```

### `build-tool build <target>`
Transpile and build a specific target.
```bash
build-tool build all
build-tool build bootloader
build-tool build kernel
```

### `build-tool make <target>`
Clean and rebuild a single target.
```bash
build-tool make bootloader
build-tool make kernel
```

### `build-tool clean`
Transpile and run `make clean`.
```bash
build-tool clean
```

### `build-tool info`
List all targets from build.toml.
```bash
build-tool info
```

## Configuration (build.toml)

### Basic Target

```toml
[targets.hello]
type = "c"
source = "hello.c"
output = "hello"
flags = "-Wall -std=c99"
```

- `type` — Language: `c`, `asm`, `rust`, `python`, `zig`, `link`
- `source` — Input file
- `output` — Output file/binary
- `flags` — Compiler flags (optional, defaults to language-specific)
- `depends_on` — Targets this depends on (optional)

### Dependencies

```toml
[targets.boot]
type = "asm"
source = "boot.s"
output = "boot.o"

[targets.kernel]
type = "c"
source = "kernel.c"
output = "kernel.o"
depends_on = ["boot.o"]

[targets.final]
type = "link"
output = "kernel.elf"
link_with = ["boot.o", "kernel.o"]
flags = "-T linker.ld"
depends_on = ["boot", "kernel"]
```

When you run `build-tool build final`:
- If `boot.s` changed → rebuild `boot.o`
- If `kernel.c` changed → rebuild `kernel.o`
- Only rebuild `kernel.elf` if dependencies changed

This is **directed compiling**: Make only rebuilds what's needed.

### Multiple Variants of Same Source

```toml
[targets.lib_release]
type = "c"
source = "lib.c"
output = "lib_release.o"
flags = "-O3 -flto"

[targets.lib_debug]
type = "c"
source = "lib.c"
output = "lib_debug.o"
flags = "-g -O0"

[targets.test]
type = "c"
source = "test.c"
output = "test"
depends_on = ["lib_debug.o"]
```

Both targets use same source with different flags. No code duplication.

## Supported Languages

### C
```toml
[targets.mylib]
type = "c"
source = "src/lib.c"
output = "lib.o"
flags = "-Wall -O2 -fPIC"
```

Compiler: `gcc` (via `$GCC` env var)

Common flags:
- `-Wall -Wextra` — Warnings
- `-O0 -O1 -O2 -O3` — Optimization levels
- `-g` — Debug symbols
- `-fPIC` — Position independent code
- `-ffreestanding -fno-builtin` — Kernel code

### Assembly (NASM)
```toml
[targets.boot]
type = "asm"
source = "boot.s"
output = "boot.o"
flags = "-f elf64"
```

Compiler: `nasm` (via `$NASM` env var)

Formats: `-f elf32`, `-f elf64`, `-f bin`

### Rust
```toml
[targets.lib]
type = "rust"
source = "Cargo.toml"
output = "target/release/mylib.a"
flags = "--release"
```

Compiler: `cargo` (via `$CARGO` env var)

Flags: `--release` (optimized), `--debug` (unoptimized)

### Python
```toml
[targets.tool]
type = "python"
source = "tool.py"
output = "tool"
flags = "--onefile"
```

Compiler: `pyinstaller` (via `$PYINSTALLER` env var)

Common flags:
- `--onefile` — Single executable
- `--windowed` — No console (GUI apps)

### Zig
```toml
[targets.program]
type = "zig"
source = "main.zig"
output = "program"
flags = "build-exe -O ReleaseFast"
```

Compiler: `zig` (via `$ZIG` env var)

### Linker
```toml
[targets.kernel]
type = "link"
output = "kernel.elf"
link_with = ["boot.o", "kernel.o"]
flags = "-T linker.ld"
depends_on = ["boot", "kernel"]
```

Compiler: `ld` (via `$LD` env var)

## Generated Makefile

Run `build-tool transpile` to see the generated Makefile:

```makefile
# Auto-generated Makefile from build.toml
# DO NOT EDIT MANUALLY
# Regenerate with: build-tool transpile

GCC ?= gcc
NASM ?= nasm
CARGO ?= cargo
LD ?= ld

KERNEL_FLAGS = -ffreestanding -fno-builtin
LINK_SCRIPT = linker.ld

.PHONY: all clean help bootloader kernel final

bootloader: boot.s
	nasm -f elf64 boot.s -o boot.o

kernel: kernel.c boot.o
	gcc -Wall -O2 -ffreestanding -fno-builtin kernel.c -c -o kernel.o

final: boot.o kernel.o
	ld -T linker.ld boot.o kernel.o -o kernel.elf

all: bootloader kernel final

clean:
	@rm -f boot.o kernel.o kernel.elf
	@cargo clean 2>/dev/null || true

help:
	@echo "Available targets:"
	@echo "  - bootloader"
	@echo "  - kernel"
	@echo "  - final"
```

The Makefile is **real** and **portable**. You can:
- Edit it manually if needed
- Share it with teammates (they don't need build-tool installed)
- Use it in CI/CD
- Run `make` directly (no build-tool needed)

## Advanced: Custom Flags at Runtime

```bash
# Use default flags
build-tool build myprogram

# Override with make directly (Makefile already generated)
make CFLAGS="-O3 -march=native" myprogram
```

## Adding New Languages

Edit `languages.toml`:

```toml
[languages.mylangs]
compiler = "myc"
compiler_var = "MYLANGS"
extension = ".ml"
default_flags = "-O2"
compile_cmd = "{compiler} {flags} {source} -o {output}"
description = "MyLanguage compiler"

[languages.mylangs.options]
"-O2" = "Optimization level 2"
"-g" = "Debug symbols"
```

Then use in `build.toml`:

```toml
[targets.program]
type = "mylangs"
source = "program.ml"
output = "program"
```

Regenerate: `build-tool transpile`

## Workflow

### Daily use (editing code)
```bash
build-tool build all          # Build everything
build-tool build bootloader   # Build one target
build-tool clean              # Clean outputs
```

### After changing build.toml
```bash
build-tool transpile          # Regenerate Makefile
make all                       # Or use build-tool build all
```

### In CI/CD
```bash
build-tool transpile
make all
```

## Troubleshooting

### Error: "Unknown language type: 'mytype'"
Check `languages.toml` for typos. Run `build-tool info` to see available targets.

### Rebuilding more than expected
Check `depends_on` in build.toml. Make rebuilds when prerequisites change.

### Compiler not found
Set environment variables:
```bash
export GCC=gcc-11
export NASM=nasm
export CARGO=cargo
build-tool build all
```

Or edit Makefile manually (it's just a file).

## Philosophy

- **TOML is the source of truth** — not a Makefile
- **Makefile is generated, real, and portable**
- **CLI-first** — no GUI bloat, works over SSH
- **Minimal code** — transpiler is ~200 lines
- **Extensible** — add languages without recompiling
- **Version control friendly** — diffs show what changed

The tool stays out of your way. It transpiles. That's it.
