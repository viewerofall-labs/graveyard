# graveyard

A practically random dump of archived projects, one-offs, and experiments that didn't make it or got superseded. No order, no theme, no promises. Updated whenever.

| Thing | What |
|-------|------|
| `bagel` | donut.c inspired with music |
| `cipher` | Cipher os game |
| `consolefetch` | Early consolefetch builds |
| `cstress` | Stress test thing |
| `exec` | Executables folder with everything |
| `gstress` | GPU stress probably |
| `maketoml` | Toml system to transpile to makefile |
| `ok` | ok typer game |
| `oneshot-world-machine` | Hyprland rice → [TWM-hyprland](https://github.com/viewerofall-labs/TWM-hyprland) |
| `picotemp` | Onboard led blinker  |
| `pitime` | Raspberry pi pico vs 100000 digits of pi |
| `shacmp` | SHA compare util maybe |
| `testinstall` | Test curl and pkgbuild things |
| `tpmenc` | TPM encryption thing |
| `webserver` | Local web server thing |
| `dice_of_death.asm` | x86 asm dice roller, presumably lethal |
| `gpu_glitch_x11.c` | X11 GPU glitch effect |
| `lat.c` | thing, if dir go to lsd, if file use bat |
| `rn.c` | Interactive rename system |
| `build.toml` | maketoml build file for here|
| `languages.toml` | language index for maketoml system here |
| `quantumclicker.zip` | Clicker game, quantum written in yourmom.|
| `devpanel` | Dead control panel system for my websites. I sorta dont care about it anymore. | 
| `launchapd` | Launchpad application that barely works and is spelt wrong. First project ever so. |
| `noah-ark-mod` | Noahs ark mod for minecraft fabric 1.20.1. Works. Check the readme. | 
| `oneshot` | oneshot themed music player, solstice theme + library swap and base theme with custom stuff |
| `powermenu` | Crappy power menu when I tried learning rust, good for when I was learning the rust book |
| `systemd-tui` | Systemd service management, no user process implant. |
| `systemfetch` | Systemfetch thingy with all the information known to man. |

> Most of this is undocumented. That's the point.

---

## Full catalog

Everything below expands on the table above. Submodules may be empty until you `git submodule update --init`. Omitted on purpose: `webapp.sh`.

### Layout

```
dead/
├── exec/                 # Prebuilt binaries (see exec/)
├── maketoml/             # build-tool crate + its own build.toml
├── yourmom-linux-v4/     # Arch ISO source for YoMom Linux
├── <project>/            # Rust/C/Java/etc. sources
├── build.toml            # Root maketoml targets → exec/
├── languages.toml        # Compiler registry for maketoml
├── Makefile              # Auto-generated from build.toml
└── *.py, *.c, *.sh, …    # Root one-offs and utilities
```

### Build system (`maketoml/`, `build.toml`, `languages.toml`, `Makefile`)

**maketoml** (`maketoml/`) is the `build-tool` crate: TOML → Makefile → `make`. Binary installs as `build-tool` or `exec/tomk`. See `maketoml/README.md` for full usage.

Root **`build.toml`** defines compile targets for this graveyard:

| Target | Type | Source | Output |
|--------|------|--------|--------|
| `lat` | c | `lat.c` | `exec/lat` |
| `gpu_glitch_x11` | c | `gpu_glitch_x11.c` | `exec/gpu_glitch_x11` |
| `bagel`, `cipher`, `cstress`, `gstress`, `ok`, `shacmp`, `tpmenc` | rust | `*/Cargo.toml` | `exec/<name>` |
| `maketoml` | rust | `maketoml/Cargo.toml` | `exec/tomk` |
| `picotemp`, `pitime` | pico | `*/Cargo.toml` | `pi/*.uf2` |

**`languages.toml`** registers compilers: C (gcc), asm (nasm), Rust (cargo), Python (pyinstaller), Zig, linker (ld), and **pico** (RP2040 via `thumbv6m-none-eabi` + `elf2uf2-rs`).

**`Makefile`** at repo root is generated — do not hand-edit; run `build-tool transpile` from `maketoml` after changing `build.toml`.

```bash
cd maketoml && cargo build --release
./target/release/build-tool transpile   # from repo root with build.toml present
make all
```

---

### Root utilities & scripts

#### `lang.py`

Codebase “weight” scanner: walks the tree, maps extensions to languages (Rust, C, `.yourmom`, `.momjoke`, shaders, etc.), prints percentage breakdown by bytes on disk.

```bash
python3 lang.py              # summary
python3 lang.py -f           # list files for top language
python3 lang.py -g           # GitHub mode (skip lockfiles, md, toml, yaml, …)
```

Skips `.git`, `target`, `node_modules`, `venv`, `__pycache__`, and other junk dirs.

#### `dirjson.py`

Directory listing → JSON for TUIs or workers. One arg: path (supports `~`).

```bash
python3 dirjson.py ~/dead
# {"files":[{"name":"bagel","path":"...","isDir":true}, ...]}
```

#### `artifact.py`

GTK3 + **GtkLayerShell** fullscreen overlay that fakes GPU failure: VRAM dot grids, horizontal tearing bands, random frame drops (~12 FPS). Namespace `gpu_artifact_sim`. Compositor overlay, not X11.

#### `debug_validate_links.py`

Temporary helper for DMS plugin registry link validation; dynamically loads `.github/validate_links.py`. Defaults to `viewerofall-*` plugin JSONs. Delete when done testing.

#### `audio-ctl.sh`

PipeWire/Pulse audio TUI via **gum** (fallback **fzf**). TWM colors (`#c792ea`, `#00e5c8`). Pick sink, set volume, move streams.

#### `aurbrowser.sh`

Fuzzy AUR installer: `yay -Slqa` piped through fzf with live `yay -Siia` preview, multi-select, `yay -S` install. Omarchy sudo keepalive hooks.

#### `Modelfile`

Ollama model definition (`qwen3:8b`) tuned for local systems-programming assistant: Rust/C/Lua/JS, Niri, AMD 6700 XT, project map (woven, veil, yourmom, overview, …).

#### `overview.tar.gz`

Archived tarball of **overview** (Niri workspace switcher, Tauri v2) — not extracted in-tree.

#### `LICENSE`

WTFPL+ (Do What The Fuck You Want To Public License Plus).

---

### Root source files (not in table)

| File | What |
|------|------|
| `panic.c` | “Kernel panic trigger” — confirms `yes`, 10s timeout, writes to `/proc/sysrq-trigger` (actually crashes the box). Built → `exec/panic`. |
| `dice_of_death.asm` | x86-64 Linux asm dice roller; includes fork-bomb path and “stroke” messaging. Lethal if you compile and run wrong. |
| `gpu_glitch_x11.c` | X11 + GLX + GLEW fullscreen shader glitch (separate from `artifact.py` Wayland overlay). |
| `lat.c` | `stat()` path → `execvp("lsd")` for dirs, `execvp("bat")` for files. |
| `rn.c` | Interactive rename; preserves extension if you omit one. |

---

### `exec/` — prebuilt binaries

Drop zone for `make` / manual builds. Not all sources live in `dead/` anymore.

| Binary | Origin / notes |
|--------|----------------|
| `bagel` | `bagel/` — terminal 3D donut + rodio audio (`music.ogg`, `wii.mp3` in crate) |
| `cipher` | `cipher/` — egui “Cipher OS” puzzle/desktop toy |
| `cstress` | `cstress/` — multi-thread CPU FP stress |
| `gstress` | `gstress/` — **wgpu** GPU compute stress |
| `hi` | `webserver/` crate name is `hi` — axum static “hi” starfield page |
| `ok` | `ok/` — ratatui “ok typer” with infinite rank titles |
| `oneshot` | `oneshot/` — OneShot-themed music player (egui + rodio) |
| `quickpower` | `powermenu/` — GTK4/libadwaita power menu (Catppuccin) |
| `sysfetch` | `systemfetch/` — verbose system info dump |
| `tpmenc` | `tpmenc/` — TPM-sealed file encrypt/decrypt |
| `lat`, `rn`, `panic` | C utilities above |
| `kys2` | Static binary; source not in tree |
| `stroke` | PIE binary; likely related to `dice_of_death.asm` “stroke” path |
| `sysd-tui-bundle-exec/` | `client`, `sysd`, `systemd-tui-daemon` — bundled systemd-tui builds |

`.rustc_info.json` — rustc fingerprint cache artifact, ignore.

---

### Projects (directories)

#### `bagel/`

Rust terminal **donut.c** clone (crossterm ASCII torus) with background music via rodio. Assets: `music.ogg`, `wii.mp3`, `src/assets/`.

#### `cipher/`

Rust + **egui** desktop “Cipher OS” — decrypt/discover puzzle UI, fake window chrome, ~800+ lines in `src/main.rs`.

#### `consolefetch/` (git submodule)

Submodule → [consolefetch](https://github.com/viewerofall/consolefetch). Early/archived consolefetch builds. Run `git submodule update --init consolefetch` to populate.

#### `cstress/`

CPU stress: spawns `num_cpus` threads, heavy floating-point loops, stats every second. No GPU.

#### `gstress/`

Real GPU stress via **wgpu** (async main): adapter request, compute shaders, sysinfo monitoring. Distinct from `cstress`.

#### `devpanel/`

Cloudflare Worker **global control panel** — banners, lockdown state, KV-backed config. `devpanel-worker.js`, `wrangler.toml`, `package.json`. Deploy: `wrangler deploy`. Abandoned-ish but functional.

#### `launchapd/`

First project ever. **iced** GUI app launcher (name misspelled on purpose). Modes: GUI, `--rofi`, `--add`, `--list`, `--remove`, `-l/--launch`. Persists app list to disk.

#### `maketoml/`

The `build-tool` package. `src/` transpiler, `build.toml.example`, local `Makefile`, own `languages.toml`. Root graveyard copies `build.toml` / `languages.toml` for monorepo builds.

#### `noah-ark-mod/`

Fabric **1.20.1** Minecraft mod (Gradle + Loom). Noah’s ark flood mechanic, slurp blocks/items, `StormManager`, `SlurplingEntity`, gopher logs, `/thenewstart` command, client HUD overlay. Texture gen: `gen_textures.py`. Works — see in-repo Fabric template README for IDE setup.

#### `ok/`

Terminal game: press ok, ranks escalate forever (tiered title generator: grounded → cosmic → unhinged → post-language). ratatui + crossterm. `mod store` for state.

#### `oneshot/`

egui music player — **Solstice** and **Base Game** modes, custom themes, library path, shuffle, background images, rodio playback, settings JSON on disk.

#### `oneshot-world-machine/` (git submodule)

Hyprland rice configs → lives at [TWM-hyprland](https://github.com/viewerofall-labs/TWM-hyprland). Submodule dir empty until init.

#### `picotemp/`

RP2040 (**rp-pico**): onboard LED patterns (solid, blink, morse, strobe, …), USB serial control. `memory.x`, `build.rs`, ships `picotemp.uf2`. Morse-over-USB firmware toy.

#### `pitime/`

RP2040: stdin asks for N decimal places of π, prints via `compute_pi` crate. UF2 via maketoml pico target.

#### `powermenu/`

GTK4 + libadwaita power menu (`quickpower` binary). Shutdown, reboot, suspend, lock, etc. Catppuccin Mocha CSS. Learning-Rust-era artifact.

#### `pwrbtn/`

Shell scripts simulating USB **power button** long/short press via `/dev/input` — `pwrbtn-long`, `pwrbtn-short`. See `pwrbtn/README.md`; needs a “Power Button” evdev node.

#### `shacmp/`

CLI + optional GUI SHA-256 compare (`hashcmp` binary name in help). Subcommands: `cmp`, watch mode, directory walks. clap + walkdir + egui for GUI path.

#### `systemd-tui/`

Workspace: `crates/daemon`, `crates/client`, `crates/shared`.

- **daemon**: Unix socket `/tmp/systemd-tui.sock`, D-Bus systemd, Lua config engine, zbus.
- **client**: ratatui TUI — list/filter services (all/active/inactive/failed), start/stop/restart via socket protocol.
- No user-session implant; system-focused.

Prebuilt copy under `exec/sysd-tui-bundle-exec/`.

#### `systemfetch/`

Binary `sysfetch`. sysinfo + whoami: OS, kernel, CPU, RAM, disks, networks, temps, uptime — boxed terminal output.

#### `testinstall/`

Install test harnesses (questionable AUR methods, documented anyway):

- **`aur-test-harness.sh`** — PKGBUILD in fake `$HOME`, logging for failures.
- **`woven-test-harness.sh`** — curl/get.sh style installers into isolated dir.

See `testinstall/README.md`.

#### `tpmenc/`

TPM2 seal/unseal files (`tpmenc seal`, `unseal`, `preview`). Machine-bound, no stored keys. Modules: `src/tpm.rs`, `src/crypto.rs`.

#### `webserver/`

Crate **`hi`**: minimal axum server serving one HTML page (starfield + “hi”). Builds to `exec/hi`.

#### `yourmom-linux-v4/`

Arch-based **YoMom Linux** ISO tree (VQ3/VQ4 era). Not the live compiler repo — ISO + baked docs.

```
yourmom-linux-v4/
└── yourmom-iso/           # mkarchiso profile
    ├── airootfs/          # rootfs overlay (passwd, motd, systemd units, …)
    ├── grub/, syslinux/, efiboot/
    ├── pacman.conf, packages.x86_64, profiledef.sh
    └── STARTUP_GUIDE.md   # build flash QEMU install flow
```

- **`/etc/os-release`**: `YoMom Linux VQ3 (Version 3: Quantum)`, `BUILD_ID=vq3`
- **`yominit-splash.service`**: boot splash on tty1 before getty/DM
- **`airootfs/usr/local/share/yourmom/`**: `README.md`, `LANGUAGE_SPEC.md`, `quantum_runtime.h`, `jksmpl.momjoke`, `qol.momjoke` — language docs/runtime shipped on ISO
- Live installer command: `join-game` (per STARTUP_GUIDE)
- Compiler built separately (`yourmom-rust` mentioned in guide; not vendored in `dead/`)

Language itself: quantum esoteric lang → C → gcc. `.yourmom` sources, `.yourdad` projects, `.momjoke` aliases, yo-mama error messages. Upstream: [yourmom-lang](https://github.com/viewerofall/yourmom-lang).

---

### Git submodules (`.gitmodules`)

| Path | URL |
|------|-----|
| `oneshot-world-machine` | https://github.com/viewerofall-labs/TWM-hyprland |
| `consolefetch` | https://github.com/viewerofall/consolefetch |

```bash
git submodule update --init --recursive
```

---

### Quick reference — everything in tree

| Path | Type |
|------|------|
| `bagel/` | Rust / terminal |
| `cipher/` | Rust / egui game |
| `consolefetch/` | submodule |
| `cstress/` | Rust / CPU stress |
| `devpanel/` | CF Worker JS |
| `exec/` | binaries |
| `gstress/` | Rust / wgpu GPU |
| `launchapd/` | Rust / iced launcher |
| `maketoml/` | Rust / build-tool |
| `noah-ark-mod/` | Java / Fabric mod |
| `ok/` | Rust / TUI game |
| `oneshot/` | Rust / egui music |
| `oneshot-world-machine/` | submodule |
| `picotemp/` | Rust / RP2040 |
| `pitime/` | Rust / RP2040 |
| `powermenu/` | Rust / GTK4 |
| `pwrbtn/` | shell scripts |
| `shacmp/` | Rust / hash tool |
| `systemd-tui/` | Rust workspace |
| `systemfetch/` | Rust / fetch |
| `testinstall/` | shell harnesses |
| `tpmenc/` | Rust / TPM crypto |
| `webserver/` | Rust / axum |
| `yourmom-linux-v4/` | Arch ISO |
| `artifact.py` | Python / GTK overlay |
| `audio-ctl.sh` | shell |
| `aurbrowser.sh` | shell |
| `build.toml` | maketoml config |
| `debug_validate_links.py` | Python |
| `dirjson.py` | Python |
| `dice_of_death.asm` | asm |
| `gpu_glitch_x11.c` | C / X11 GL |
| `lang.py` | Python |
| `languages.toml` | maketoml registry |
| `lat.c` | C |
| `Makefile` | generated |
| `Modelfile` | Ollama |
| `overview.tar.gz` | archive |
| `panic.c` | C |
| `rn.c` | C |
| `LICENSE` | license |

---

### Building common targets

```bash
# Single Rust project
cargo build --release --manifest-path bagel/Cargo.toml

# Graveyard-wide (after maketoml built)
make -C /home/abyss/dead all

# Pico UF2
make -C /home/abyss/dead picotemp pitime

# YoMom ISO (on Arch, needs archiso + separate yourmom compiler build)
sudo mkarchiso -v -w /tmp/yourmom-work -o /tmp/yourmom-out yourmom-linux-v4/yourmom-iso/
```

---

### Live / active replacements

Stuff here was superseded or forked elsewhere:

| Dead | Alive |
|------|-------|
| `consolefetch/` | `~/consolefetch` |
| `oneshot-world-machine` | [TWM-hyprland](https://github.com/viewerofall-labs/TWM-hyprland) |
| `systemfetch` | fastfetch / other fetchers |
| `maketoml` | may still be useful standalone |
| `yourmom-linux-v4` | `~/yourmom` / `yomama-linux` lineage |
| `overview.tar.gz` | `~/overview` |

---

> Expanded catalog. Table at top is still the source of truth for one-liners.
