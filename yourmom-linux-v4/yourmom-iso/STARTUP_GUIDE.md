# YoMom Linux — VQ4 Startup Guide

**"btw I use yo mama"**

---

## What's in the Package

```
yourmom-iso/    — Arch-based custom Linux ISO source (build with mkarchiso)
yourmom-rust/   — The .yourmom language compiler (Rust)
```

---

## 1. Build the Compiler

You need this before building the ISO so the binary gets baked in.

```bash
cd yourmom-rust
cargo build --release
```

Binary ends up at `yourmom-rust/target/release/yourmom`.

**Requirements:** `rustc`, `cargo`, `gcc`

---

## 2. Build the ISO

```bash
# Install archiso if you don't have it
sudo pacman -S archiso

# Build the ISO (must be run as root)
sudo mkarchiso -v -w /tmp/yourmom-work -o /tmp/yourmom-out yourmom-iso/
```

Output ISO will be at `/tmp/yourmom-out/yourmom-*.iso`

**Requirements:** Arch Linux (or Arch-based), `archiso`

> If you're not on Arch, you can run mkarchiso inside an Arch container/VM.

---

## 3. Flash to USB

```bash
sudo dd if=/tmp/yourmom-out/yourmom-*.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Replace `/dev/sdX` with your USB drive (check with `lsblk`).

Or use a GUI tool like Ventoy, Etcher, or KDE ISO Image Writer.

---

## 4. Test in a VM (QEMU)

```bash
# UEFI boot
qemu-system-x86_64 -enable-kvm -m 4G -cdrom /tmp/yourmom-out/yourmom-*.iso \
    -bios /usr/share/ovmf/x64/OVMF.fd -boot d

# BIOS boot
qemu-system-x86_64 -enable-kvm -m 4G -cdrom /tmp/yourmom-out/yourmom-*.iso -boot d
```

---

## 5. Install the OS

Once booted into the live environment, run:

```bash
join-game
```

The installer will walk you through:
- Disk selection
- Filesystem (ext4, btrfs, xfs)
- Timezone & locale
- Hostname & user setup
- Desktop environment (optional)
- Bootloader (GRUB, systemd-boot, rEFInd)
- Extra packages (searchable with `/`)

---

## 6. Package Management

YoMom Linux uses themed wrappers around `pacman` and `yay`:

### `mama` — Official packages (pacman)

```bash
mama get firefox          # install
mama yeet firefox         # remove
mama upgrade              # update everything
mama search neovim        # search
mama info htop            # package details
mama list                 # list installed
mama clean                # clear cache
```

### `dada` — AUR packages (yay)

```bash
dada get discord          # install from AUR
dada yeet discord         # remove
dada upgrade              # update AUR packages
dada search spotify       # search AUR
```

> `dada` requires `yay` — install it first with `mama get yay` or build from AUR manually.

---

## 7. Service Management

```bash
yominit start nginx         # start a service
yominit stop nginx          # stop a service
yominit restart nginx       # restart
yominit status              # show all service status
yominit enable sshd         # enable at boot
yominit disable sshd        # disable
yominit list                # running services
yominit list-all            # all services
yominit log -f nginx        # follow logs
yominit failed              # show failed services
yominit reboot              # reboot
yominit poweroff            # power off
```

---

## 8. The .yourmom Language

Full docs are in `yourmom-rust/README.md`. Quick rundown:

```bash
# Compile a .yourmom file
yourmom childmake hello.yourmom

# Compile and run immediately
yourmom run hello.yourmom

# Clean build artifacts
yourmom abortion

# Show dependency tree (.yourdad)
yourmom family-tree project.dad
```

### Hello World

```yourmom
yo mama_main() {
    ymf("Hello, World!")
}
```

### Quantum Superposition

```yourmom
yo x = 1 | 2 | 3       // collapses to one value on first use
ymf(x)                  // e.g. 2
ymf(x)                  // same — 2
```

### Heisenberg Variable

```yourmom
yo h = heisenberg(10 | 20 | 30)
ymf(h)    // random every time
ymf(h)    // different random value
```

---

## 9. Project Structure Quick Reference

```
yourmom-iso/
├── profiledef.sh           — ISO metadata, file permissions
├── packages.x86_64         — Packages installed in the ISO
├── pacman.conf             — Pacman config for the build
├── airootfs/
│   ├── etc/
│   │   ├── motd            — Message of the day
│   │   └── systemd/system/ — Custom service units
│   └── usr/local/bin/
│       ├── join-game       — Installer TUI
│       ├── mama            — pacman wrapper
│       ├── dada            — yay/AUR wrapper
│       ├── yominit         — systemd wrapper
│       ├── yominit-splash  — Boot splash
│       └── yourmom         — Language compiler binary
yourmom-rust/
├── src/                    — Compiler source
├── Cargo.toml
└── *.yourmom               — Example programs
```

---

## 10. Common Issues

**ISO build fails with "package not found"**
Make sure your mirrorlist is up to date: `sudo reflector --save /etc/pacman.d/mirrorlist --latest 10`

**`join-game` crashes during install**
Run with bash tracing to see where: `bash -x /usr/local/bin/join-game`

**`dada` says yay not found**
Install yay first: build from source or `mama get yay` if it's in the repos.

**Compiler can't find gcc**
`mama get gcc` — GCC is required to produce binaries from .yourmom files.

---

## Credits

Original concept by **viewerofall**.
VQ4 — built on Arch, compiled in Rust, themed in yo mama jokes.

*WTFPL — Do what the fuck you want.*
