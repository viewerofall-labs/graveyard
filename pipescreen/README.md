# pipescreen

Idle-triggered **pipes** screensaver + auto-suspend for Niri/Wayland. One Zig
binary, no libwayland, no scanner — it speaks `ext-idle-notify-v1` over the
wire directly.

- **5 min idle** → fullscreen kitty drawing pipes (rendered in Zig, themed to
  the abyss palette).
- **15 min idle** → `systemctl suspend`.
- **any input** → screensaver killed.

```
pipescreen                 # daemon: watch idle, drive the screensaver
pipescreen render          # just draw pipes (what the daemon spawns in a fullscreen kitty)

# control a running daemon (over $XDG_RUNTIME_DIR/pipescreen.sock):
pipescreen trigger         # show the screensaver right now
pipescreen stop            # hide it
pipescreen awake [on|off]  # keep-awake: inhibit pipes AND suspend (no arg = toggle)
pipescreen pipes  [on|off] # enable/disable the screensaver
pipescreen suspend [on|off]# enable/disable auto-suspend
pipescreen lock    [on|off]# enable/disable locking (woven-lock) before suspend
pipescreen status          # awake=.. pipes=.. suspend=.. lock=.. showing=..
```

When `lock` is on, the daemon runs `woven-lock` (a moment before `systemctl
suspend`) so the session is locked on resume. `woven-lock` takes no args and
releases on exit; change `LOCK_CMD`/`LOCK_SETTLE_MS` at the top of
`src/main.zig` to use a different locker or settle time.

Pipes spawn with a randomized count/position each cycle. The screensaver is
dismissed by any input (a short internal idle-notification makes resume snappy),
or with `pipescreen stop`.

## Build & install

```sh
zig build -Doptimize=ReleaseFast
install -Dm755 zig-out/bin/pipescreen ~/.local/bin/pipescreen
```

## Run it

**Niri (simplest)** — inherits the session env automatically. In `config.kdl`:

```kdl
spawn-at-startup "pipescreen"
```

**systemd --user** — runs as your user, never root, scoped to the graphical
session:

```sh
install -Dm644 systemd/pipescreen.service ~/.config/systemd/user/pipescreen.service
# Niri must hand the wayland socket to the user manager once per login:
systemctl --user import-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR
systemctl --user enable --now pipescreen.service
```

(If using the systemd path, add the `import-environment` line to a
`spawn-at-startup` in niri so it runs on every login.)

## Tuning

Knobs live at the top of `src/main.zig`:

```zig
const PIPES_MS:   u32 = 5 * 60 * 1000;   // screensaver delay
const SUSPEND_MS: u32 = 15 * 60 * 1000;  // suspend delay
const SUSPEND_ENABLED = true;            // false = screensaver only
```

Pipe colors / count / speed live at the top of `src/pipes.zig`.

## Test the timers without waiting 5 minutes

Drop `PIPES_MS` to `5000` and set `SUSPEND_ENABLED = false`, rebuild, run
`pipescreen`, then don't touch the mouse/keyboard for ~6s — a fullscreen kitty
full of pipes should appear, and vanish the moment you move.

## Layout

```
src/main.zig    arg dispatch + daemon (spawn/kill saver, suspend)
src/idle.zig    hand-rolled ext-idle-notify-v1 wire client (raw linux syscalls)
src/pipes.zig   the pipes renderer (ANSI + box-drawing)
```
