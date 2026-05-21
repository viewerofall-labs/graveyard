# .yourmom Language Specification

**Version:** VQ 3.0 (Version Quantum)
**Tagline:** "Holy C but what the fuck happened"
**Creator:** viewerofall

---

## Table of Contents

1. [Overview](#overview)
2. [Compilation Architecture](#compilation-architecture)
3. [Syntax Reference](#syntax-reference)
4. [Quantum Mechanics System](#quantum-mechanics-system)
5. [Advanced Features](#advanced-features)
6. [Standard Library](#standard-library)
7. [CLI Commands](#cli-commands)
8. [Build System (.yourdad)](#build-system)
9. [Custom Shortcuts (.momjoke)](#custom-shortcuts)
10. [C Interoperability](#c-interoperability)
11. [Error Messages](#error-messages)
12. [Implementation Details](#implementation-details)

---

## Overview

.yourmom is a quantum-enabled esoteric programming language that transpiles to C via GCC. It provides:

- Quantum superposition, entanglement, and Heisenberg uncertainty as first-class language features
- Eight number type categories including imaginary, quaternion, and transfinite numbers
- A family-themed CLI (`childmake`, `abortion`, `divorce`, `deadbeat`, `custody`)
- Automatic C library detection and linking
- Your mom jokes as compile errors and runtime panics

Despite the humor, it compiles to real native binaries with C-level performance.

---

## Compilation Architecture

```
.yourmom source file
    ↓
Lexer — expands .momjoke shortcuts inline, tracks line/column positions
    ↓
Parser — builds AST, emits Rust-style diagnostics with source context
    ↓
Codegen — walks AST, emits C code
    ↓
Auto-detection — scans generated C for known library function names
    ↓
pkg-config — resolves cflags/ldflags for detected libraries
    ↓
Header injection — prepends #include directives into generated C
    ↓
GCC — compiles to native binary
```

**Compiler source layout:**

```
src/
├── main.rs           CLI entry point (clap)
├── lib.rs            Public API: compile_to_c, compile_to_binary
├── lexer/
│   ├── mod.rs        Tokenizer with line/column tracking
│   ├── token.rs      TokenKind enum, Token struct, Span, Position
│   └── shortcuts.rs  .momjoke shorthand expansion
├── parser/
│   ├── mod.rs        Recursive descent parser
│   └── ast.rs        AST node types
├── codegen/
│   └── mod.rs        C code emitter
├── errors/
│   └── mod.rs        YourMomError, Diagnostic, yo mama joke database
├── build_system/
│   └── mod.rs        .yourdad parser, auto-library detection
├── cli/
│   └── mod.rs        Command implementations
└── runtime/
    ├── mod.rs         Embeds quantum_runtime.h at compile time
    └── quantum_runtime.h  Full quantum C runtime (pure header)
```

---

## Syntax Reference

### Variables

```yourmom
yo x = 42             // integer
yo name = "hello"     // string
yo pi = 3.14          // float
```

Assignment without `yo` re-assigns an existing variable:

```yourmom
yo x = 0
x = x + 1
```

### Functions

```yourmom
yo add(a, b) {
    did_your_mom a + b
}

yo mama_main() {
    yo result = doing_your_mom add(10, 20)
    ymf(result)
}
```

- **Entry point:** `mama_main()` (equivalent to C `main`)
- **Return:** `did_your_mom <value>`
- **Call expression:** `doing_your_mom <name>(args)` or shorthand `dym <name>(args)`
- **Print:** `yo_mama_so_fat(value)` or shorthand `ymf(value)`

### Control Flow

**If / else:**
```yourmom
mama_said x > 0 {
    ymf("positive")
} mama_lied {
    ymf("not positive")
}
```

**While:**
```yourmom
mama_keeps_saying x > 0 {
    x = x - 1
}
```

**Goto:**
```yourmom
mama_forgot my_label
ymf("this is skipped")
my_label:
    ymf("execution jumps here")
```

**Schrödinger's if** — ignores condition, flips quantum coin:
```yourmom
mama_said_maybe {
    ymf("branch A")
} mama_lied {
    ymf("branch B")
}
```

### Try / Catch

```yourmom
yo_daddy_issues {
    ymf("risky code")
} mama_caught {
    ymf("something went wrong")
}
```

Uses C `setjmp`/`longjmp` under the hood.

### Operators

**Arithmetic:** `+`, `-`, `*`, `/`, `%`
**Comparison:** `==`, `!=`, `<`, `>`, `<=`, `>=`
**Quantum separator:** `|` (superposition values)
**Unary minus:** `-42` works as a literal

### Comments

```yourmom
// single-line comment only
```

---

## Quantum Mechanics System

### Basic Superposition

```yourmom
yo x = 10 | 20 | 30 | 40
ymf(x)    // collapses to one value, e.g. 20
ymf(x)    // same value — already collapsed
ymf(x)    // still 20
```

First read triggers collapse (random selection, seeded RNG). All subsequent reads return the same cached value.

### Number Type Superpositions

#### `int_all` — all 32-bit integers
```yourmom
yo x = int_all
ymf(x)    // e.g. -847291043
```

#### `real_all` — all double-precision floats
```yourmom
yo x = real_all
ymf(x)    // e.g. 1.7976931348623e+308
```

#### `rational_all` — rational numbers (reduced fractions)
```yourmom
yo x = rational_all
ymf(x)    // e.g. 355/113
```
Collapses to `p/q` where `gcd(p, q) = 1`.

#### `irrational_all` — famous irrational constants
```yourmom
yo x = irrational_all
ymf(x)    // e.g. pi ~= 3.14159265358979
```

Available constants: π, e, √2, √3, √5, √7, φ (golden ratio), ln(2), π/2, γ (Euler-Mascheroni), ζ(3) (Apéry), log₁₀2, δ (Feigenbaum), ∛36, ρ (plastic number)

#### `imaginary_all` — complex numbers
```yourmom
yo x = imaginary_all
ymf(x)    // e.g. 3.5+2.1i
```
Collapses to `a+bi` where `b ≠ 0`.

#### `quaternion_all` — quaternions
```yourmom
yo q = quaternion_all
ymf(q)    // e.g. 1.2+3.4i+5.6j-7.8k
```
Collapses to `w+xi+yj+zk`.

#### `transfinite_all` — infinite cardinals and ordinals
```yourmom
yo t = transfinite_all
ymf(t)    // e.g. omega^omega
```

Available: ℵ₀, ℵ₁, ℵ₂, ω, ω+1, ω², ω^ω, ε₀

### Heisenberg Variables

Re-collapses to a new random value **every time** it is read:

```yourmom
yo h = heisenberg(10 | 20 | 30)
ymf(h)    // might print 10
ymf(h)    // might print 30
ymf(h)    // might print 20
```

### Entangled Variables

Always collapses to the same value as another variable:

```yourmom
yo x = 1 | 2 | 3
yo y = entangled_with x
ymf(x)    // e.g. 2
ymf(y)    // also 2, guaranteed
```

### Quantum Sort

Sorts a superposition and collapses to the minimum:

```yourmom
yo s = quantum_sort(5 | 3 | 1 | 4 | 2)
ymf(s)    // prints: [1, 2, 3, 4, 5] -> 1
```

### Quantum Introspection

```yourmom
yo_mama_observe(x)      // force collapse without printing
yo_mama_collapsed(x)    // returns 1 if already collapsed, 0 if not
```

---

## Advanced Features

### Panic

```yourmom
yo_mama_so_dumb("something went very wrong")
```

Prints the message and calls `abort()`. Shorthand: `ymd("message")`.

Can also take an expression:
```yourmom
ymd(error_code)
```

### Loud Print

```yourmom
yo_mama_so_loud("hello world")
// Output: 🔊 BITCH: HELLO WORLD
```

### Paranoid Assertion

50% chance of actually checking the assertion at runtime:

```yourmom
yo_mama_so_paranoid(x > 0)
```

If the coin flip decides to check and the assertion fails: `PARANOID SHIT FAILED`.

### Lazy Evaluation

Value is only computed on first access, then cached:

```yourmom
yo x = yo_mama_so_lazy(100 + 200)
ymf("not computed yet")
ymf(x)    // computed here → 300
ymf(x)    // cached → 300
```

### Macros

Compile-time text substitution:

```yourmom
yo_mama_so_meta my_print(x) = ymf(x)

yo mama_main() {
    my_print("hello from macro")
}
```

### Self-Modifying Source

Rewrites its own source file at runtime:

```yourmom
yo_mama_so_meta rewrite_self("my_program.yourmom")
```

Prepends `// yo mama so meta, she rewrote herself` to the source file on disk.

### Unsafe / Inline Assembly

```yourmom
yo_mama_so_ugly {
    __asm__("nop")
}
```

Must contain at least one `__asm__` statement. Shorthand: `ymsu { ... }`.

---

## Standard Library

Defined in `jksmpl.momjoke`. Automatically loaded by the compiler.

### Core Shortcuts

| Shorthand | Full form | Description |
|-----------|-----------|-------------|
| `ymf` | `yo_mama_so_fat` | Print with newline |
| `ymd` | `yo_mama_so_dumb` | Panic / abort |
| `ymsu` | `yo_mama_so_ugly` | Unsafe block |
| `dym` | `doing_your_mom` | Function call |
| `ret` | `did_your_mom` | Return |

### I/O

| Function | Description |
|----------|-------------|
| `yo_mama_so_fat(x)` | Print value with newline |
| `yo_mama_so_loud(s)` | Print uppercase with 🔊 emoji |
| `yo_mama_so_nosy()` | Read integer from stdin |
| `yo_mama_so_quiet()` | Read string from stdin |

### Math

| Function | Description |
|----------|-------------|
| `yo_mama_so_random()` | Random integer |
| `yo_mama_so_round(x)` | Round to nearest integer |
| `yo_mama_so_square(x)` | Square (x²) |
| `yo_mama_so_thicc(x)` | Absolute value |
| `yo_mama_so_wide(x)` | Square root |

### Strings

| Function | Description |
|----------|-------------|
| `yo_mama_so_long(s)` | String length |
| `yo_mama_so_cheap_copy(dst, src)` | String copy |
| `yo_mama_so_connected(dst, src)` | String concatenation |

### Memory

| Function | Description |
|----------|-------------|
| `yo_mama_so_fat_malloc(size)` | Allocate memory |
| `yo_mama_so_cheap_free(ptr)` | Free memory |

### Quantum

| Function | Description |
|----------|-------------|
| `heisenberg(vals)` | Create Heisenberg variable |
| `entangled_with var` | Create entangled variable |
| `quantum_sort(vals)` | Sort superposition, collapse to minimum |
| `yo_mama_observe(var)` | Force collapse |
| `yo_mama_collapsed(var)` | Check if already collapsed |

---

## CLI Commands

```bash
yourmom childmake <file.yourmom>          # Compile to binary
yourmom childmake <file.yourmom> -o name  # Compile with custom output name
yourmom childmake <file.yourmom> -l lib   # Compile with extra library
yourmom childmake <project.yourdad>       # Build from project file
yourmom run <file.yourmom>                # Compile and run
yourmom abortion                          # Remove generated .c files
yourmom abortion --deep                   # Deep clean (all generated artifacts)
yourmom divorce <binary>                  # Remove a specific binary
yourmom family-tree <project.yourdad>     # Print dependency tree
yourmom deadbeat <project.yourdad>        # Find unused declared dependencies
yourmom custody list                      # List active .momjoke files
yourmom custody add <file.momjoke>        # Add a .momjoke file
yourmom custody remove <file.momjoke>     # Remove a .momjoke file
```

### Command Etymology

| Command | What it does | The joke |
|---------|-------------|----------|
| `childmake` | Compile | Daddy knocked up the source code |
| `abortion` | Clean | Abort the build artifacts |
| `divorce` | Remove binary | Sever ties with the executable |
| `family-tree` | Dep graph | Show family relationships |
| `deadbeat` | Unused deps | Find deadbeat dependencies |
| `custody` | Manage libs | Custody battle over .momjoke files |

---

## Build System

### .yourdad Format

Project configuration file (like Cargo.toml):

```yourdad
daddy_says_build_this {
    sources: ["main.yourmom", "lib.yourmom"]
    output: "my_app"
    optimization: 2
    debug: false
    warnings: true
}

daddy_says_use_protection {
    c_libs: ["pthread"]
}

daddy_says_wear_these_hats {
    "jksmpl.momjoke"
    "my_utils.momjoke"
}

daddy_left {
    unsafe_optimizations: true
}
```

### Sections

- **`daddy_says_build_this`** — source files, output name, optimization level, debug flag
- **`daddy_says_use_protection`** — C libraries to link (explicit, in addition to auto-detected)
- **`daddy_says_wear_these_hats`** — .momjoke files to load
- **`daddy_left`** — optional/excluded features

### Auto Library Detection

When compiling, the compiler scans generated C code for known function names and automatically detects which libraries are needed. It then runs `pkg-config` for proper include paths and linker flags, and injects the required `#include` directives into the generated C file before passing it to GCC.

**No manual linking required for:**
raylib, SDL2, SDL2_mixer, SDL2_image, SDL2_ttf, GLFW, OpenGL, GLEW, pthread, libcurl, GTK+3, ncurses, OpenAL, libpng, zlib, sqlite3, OpenSSL, Vulkan, ALSA, X11, Wayland, Cairo, FreeType

Example: if your `.yourmom` code calls `InitWindow(...)`, the compiler detects raylib, runs `pkg-config raylib --cflags --libs`, injects `#include <raylib.h>`, and links with the correct flags — automatically.

Libraries declared in `.yourdad` under `daddy_says_use_protection` or passed with `-l` are skipped during auto-detection (assumed already handled).

---

## Custom Shortcuts (.momjoke)

`.momjoke` files are plain-text alias tables loaded by the lexer. Any shorthand encountered in source is expanded before tokenization.

**Format:**
```
# comment
short_name -> full_name
```

Example `my_utils.momjoke`:
```
prnt -> yo_mama_so_fat
panic -> yo_mama_so_dumb
sq -> yo_mama_so_square
```

**Loading:**
```bash
# Via CLI
yourmom childmake file.yourmom

# Via .yourdad
daddy_says_wear_these_hats {
    "my_utils.momjoke"
}

# Via custody
yourmom custody add my_utils.momjoke
```

The compiler always loads `jksmpl.momjoke` first (the standard library), then any additional files in order.

---

## C Interoperability

### Calling External C Functions

Any C function can be called directly using the call syntax. The auto-detection system handles includes and linking for known libraries. For unknown libraries, use the `daddy_says_use_protection` section in `.yourdad` or the `-l` flag.

```yourmom
yo mama_main() {
    // If raylib is installed, this just works — auto-detected and linked
    dym InitWindow(800, 600, "My App")
    mama_keeps_saying dym WindowShouldClose() == 0 {
        dym BeginDrawing()
        dym ClearBackground(0, 0, 0, 255)
        dym DrawText("yo mama", 100, 100, 20, 255, 255, 255)
        dym EndDrawing()
    }
    dym CloseWindow()
}
```

### Inline Assembly

Use `yo_mama_so_ugly` blocks for raw assembly or unsafe C:

```yourmom
yo_mama_so_ugly {
    __asm__("nop")
    __asm__("mov $42, %rax")
}
```

---

## Error Messages

The compiler emits Rust-style diagnostics with source context and yo mama jokes.

**Parse error:**
```
💀 ERROR: expected `}`, found end of file
  ┌─ hello.yourmom:5:18
  │
5 │ yo mama_main() {
  │                ^ yo mama so dumb, she forgot to close this brace
  │
  = help: add `}` at the end of the function
  = note: yo mama so forgetful, she left the block wide open
```

**File not found:**
```
💀 yo mama so lost, she can't find file: foo.yourmom
   hint: check if foo.yourmom exists
```

**GCC failure:**
```
💀 yo mama so broke, GCC refused to compile
   hint: check output.c for C compilation errors
```

**Runtime panic (yo_mama_so_dumb):**
```
💀 yo mama so dumb: something went very wrong
   [process aborted]
```

**Segfault:**
```
💀 WHAT IN THE ACTUAL FUCK (signal 11):
Yo mama so fat, her ass caused a goddamn stack overflow
```

---

## Implementation Details

### Quantum Runtime (C)

The runtime is a pure C header (`quantum_runtime.h`) embedded in the compiler binary at compile time via `include_str!`. It is written to disk alongside the generated C file before GCC runs.

**Superposition struct:**
```c
typedef struct {
    int* values;
    int  count;
    int  collapsed_idx;   // -1 = not yet collapsed
    int  collapsed_val;
} quantum_int;
```

**Collapse:**
```c
int _quantum_observe(quantum_int* q) {
    if (q->collapsed_idx == -1) {
        _quantum_init_rng();
        q->collapsed_idx = rand() % q->count;
        q->collapsed_val = q->values[q->collapsed_idx];
    }
    return q->collapsed_val;
}
```

**Heisenberg (no caching):**
```c
typedef struct {
    int* values;
    int  count;
} quantum_heisenberg;

int _heisenberg_observe(quantum_heisenberg* h) {
    _quantum_init_rng();
    return h->values[rand() % h->count];
}
```

### Codegen Examples

| .yourmom | Generated C |
|----------|-------------|
| `yo x = 42` | `int x = 42;` |
| `yo x = 10 \| 20 \| 30` | `quantum_int x = SUPERPOSITION(10, 20, 30);` |
| `yo h = heisenberg(1 \| 2 \| 3)` | `quantum_heisenberg h = HEISENBERG(1, 2, 3);` |
| `ymf(x)` | `yo_mama_so_fat(x);` |
| `mama_said x > 0 { ... }` | `if (x > 0) { ... }` |
| `mama_keeps_saying x > 0 { ... }` | `while (x > 0) { ... }` |
| `mama_forgot label` | `goto label;` |
| `did_your_mom x` | `return x;` |

### Performance

| Operation | Cost |
|-----------|------|
| Regular variable access | C speed |
| Quantum collapse (first read) | ~1μs |
| Quantum read (after collapse) | Cached, ~0 overhead |
| Heisenberg read | ~1μs per read |
| Compilation (transpile only) | ~1ms |
| GCC (typical program) | ~100ms |

**Memory:**
- Regular `int`: 4 bytes
- `quantum_int`: 24 bytes + value array
- `quantum_heisenberg`: 16 bytes + value array

---

## Feature Matrix

| Feature | .yourmom | C | Rust | Python |
|---------|----------|---|------|--------|
| Quantum superposition | ✅ | ❌ | ❌ | ❌ |
| Heisenberg variables | ✅ | ❌ | ❌ | ❌ |
| Transfinite number types | ✅ | ❌ | ❌ | ❌ |
| Auto C library detection | ✅ | ❌ | ❌ | ❌ |
| Goto | ✅ | ✅ | ❌ | ❌ |
| Try/catch | ✅ | ❌ | ✅ | ✅ |
| Self-modifying source | ✅ | ❌ | ❌ | ✅ |
| Compiles to native binary | ✅ | ✅ | ✅ | ❌ |
| Your mom jokes | ✅ | ❌ | ❌ | ❌ |

---

## License

**WTFPL** — Do What The Fuck You Want To Public License

*"I'm not your mom. Do whatever you want."*
