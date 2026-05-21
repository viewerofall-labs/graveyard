# .yourmom — VQ 3.0 (Version Quantum)

> *"Holy C but what the fuck happened"*

A quantum-enabled esoteric language that transpiles to C. Your mom jokes are the error messages. The CLI is a family therapy session.

---

## Install

```bash
git clone https://github.com/viewerofall/yourmom-lang
cd yourmom-lang
cargo build --release
# Add ./target/release/yourmom to your PATH
```

## Documents

> This thing is VERY big so watch out

Full language breakdown
[Full language breakdown](./LANGUAGE_SPEC.md)
Library overview and what they do
[Libraries](./Libraries.md)
---

## Quick Start

```bash
# Compile a .yourmom file → binary
yourmom childmake hello.yourmom

# Compile and run immediately
yourmom run hello.yourmom

# Compile with custom output name
yourmom childmake hello.yourmom -o hello

# Clean build artifacts
yourmom abortion

# Project-based build (.yourdad file)
yourmom childmake project.yourdad
```

---

## Hello World

```yourmom
yo mama_main() {
    ymf("Hello, World!")
}
```

`mama_main` is the entry point. `ymf` is print. That's it.

---

## What Makes This Different

| Feature | Description |
|---------|-------------|
| **Quantum superposition** | `yo x = 1 \| 2 \| 3` — collapses to one value on first read, stays there |
| **Heisenberg variables** | Re-collapse to a new value *every single read* |
| **Entangled variables** | Two variables that always share the same collapsed value |
| **8 number types** | int, real, rational, irrational, imaginary/complex, quaternion, transfinite |
| **Schrödinger's if** | Ignores its own condition, flips a quantum coin instead |
| **Auto library detection** | Uses function names in your code to auto-detect and link C libraries |
| **`.yourdad` build system** | Like Cargo.toml but cursed |
| **`.momjoke` shorthand** | Alias system for stdlib functions |
| **Your mom joke errors** | Every compile error and crash is a yo mama joke |

---

## Language at a Glance

```yourmom
yo mama_main() {

    // Regular variable
    yo x = 42
    ymf(x)

    // Quantum superposition — collapses on first read
    yo coin = 0 | 1
    mama_said coin == 1 {
        ymf("heads")
    } mama_lied {
        ymf("tails")
    }

    // Heisenberg — different every time
    yo h = heisenberg(10 | 20 | 30)
    ymf(h)    // e.g. 20
    ymf(h)    // e.g. 10

    // All number types
    yo a = int_all
    yo b = rational_all      // prints e.g. "355/113"
    yo c = irrational_all    // prints e.g. "pi ~= 3.14159265358979"
    yo d = imaginary_all     // prints e.g. "3.5+2.1i"
    yo q = quaternion_all    // prints e.g. "1.2+3.4i+5.6j-7.8k"
    yo t = transfinite_all   // prints e.g. "omega^omega"

    // Schrödinger's if — 50/50 regardless of condition
    mama_said_maybe {
        ymf("branch A")
    } mama_lied {
        ymf("branch B")
    }

    // Try/catch
    yo_daddy_issues {
        ymf("living dangerously")
    } mama_caught {
        ymf("caught the drama")
    }

    // Lazy evaluation
    yo lazy = yo_mama_so_lazy(100 + 200)
    ymf(lazy)    // computed here, cached after

    // goto
    mama_forgot skip_this
    ymf("this is skipped")
    skip_this:
        ymf("teleported here")

}
```

---

## CLI Commands

```bash
yourmom childmake <file>      # Compile .yourmom or .yourdad
yourmom run <file>            # Compile and run
yourmom abortion              # Clean generated C files
yourmom abortion --deep       # Deep clean
yourmom divorce <binary>      # Remove a binary
yourmom family-tree <.dad>    # Dependency tree
yourmom deadbeat <.dad>       # Find unused dependencies
yourmom custody list          # List loaded .momjoke files
yourmom custody add <file>    # Add .momjoke
yourmom custody remove <file> # Remove .momjoke
```

---

## Project Files

| File | Purpose |
|------|---------|
| `*.yourmom` | Source files |
| `*.yourdad` / `*.dad` | Project config (sources, libs, momjoke imports) |
| `*.momjoke` | Shorthand alias tables |
| `quantum_runtime.h` | C runtime (embedded in compiler, also available standalone) |
| `jksmpl.momjoke` | Standard library shortcuts |

---

## How It Works

```
.yourmom source
    ↓ Lexer (expands .momjoke shortcuts)
    ↓ Parser (builds AST)
    ↓ Codegen (emits C)
    ↓ Auto-detects needed libraries (pkg-config)
    ↓ GCC
native binary
```

The quantum runtime is a pure C header (`quantum_runtime.h`) with no external dependencies. Superposition uses lazy collapse via `rand()`. Heisenberg skips the collapse cache. Try/catch uses `setjmp`/`longjmp`.

---

## Error Messages

Rust-style diagnostics with yo mama jokes:

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

---

## License

WTFPL — Do what the fuck you want. I'm not your mom.

**Creator:** viewerofall
**Influences:** HolyC, Brainfuck, quantum computing, Cookie Clicker

*See `LANGUAGE_SPEC.md` for the full language reference.*
