//! The screensaver itself: BBS/Windows-style "pipes" drawn straight to the
//! terminal with ANSI escapes. Runs until it receives SIGTERM/SIGINT (which
//! the daemon sends on resume), then restores the terminal.

const std = @import("std");
const linux = std.os.linux;

const TIOCGWINSZ = 0x5413;

const Winsize = extern struct {
    row: u16,
    col: u16,
    xpix: u16,
    ypix: u16,
};

// up, right, down, left
const UP = 0;
const RIGHT = 1;
const DOWN = 2;
const LEFT = 3;

const Rgb = struct { r: u8, g: u8, b: u8 };

// abyss palette: magenta / cyan lead, plus a couple of cool accents.
const palette = [_]Rgb{
    .{ .r = 0xc7, .g = 0x92, .b = 0xea }, // #c792ea magenta
    .{ .r = 0x00, .g = 0xe5, .b = 0xc8 }, // #00e5c8 cyan
    .{ .r = 0x82, .g = 0xaa, .b = 0xff }, // soft blue
    .{ .r = 0xf0, .g = 0x7f, .b = 0xd4 }, // pink
    .{ .r = 0x9d, .g = 0x7c, .b = 0xd8 }, // violet
};

const Pipe = struct {
    x: i32,
    y: i32,
    dir: u8,
    color: Rgb,
};

var running: bool = true;

fn onSignal(_: linux.SIG) callconv(.c) void {
    running = false;
}

fn termSize() Winsize {
    var ws: Winsize = .{ .row = 24, .col = 80, .xpix = 0, .ypix = 0 };
    _ = linux.ioctl(1, TIOCGWINSZ, @intFromPtr(&ws));
    if (ws.row == 0) ws.row = 24;
    if (ws.col == 0) ws.col = 80;
    return ws;
}

fn out(bytes: []const u8) void {
    var off: usize = 0;
    while (off < bytes.len) {
        const n = linux.write(1, bytes[off..].ptr, bytes.len - off);
        if (@as(isize, @bitCast(n)) <= 0) return;
        off += n;
    }
}

fn sleepMs(ms: u64) void {
    var req = linux.timespec{ .sec = @intCast(ms / 1000), .nsec = @intCast((ms % 1000) * std.time.ns_per_ms) };
    _ = linux.nanosleep(&req, null);
}

/// Heavy box-drawing glyph for a cell entered moving `old`, leaving `new`.
fn piece(old: u8, new: u8) []const u8 {
    if (old == new) {
        return if (old == UP or old == DOWN) "┃" else "━";
    }
    return switch (old) {
        UP => if (new == RIGHT) "┏" else "┓",
        DOWN => if (new == RIGHT) "┗" else "┛",
        RIGHT => if (new == UP) "┛" else "┓",
        LEFT => if (new == UP) "┗" else "┏",
        else => "╋",
    };
}

fn draw(x: i32, y: i32, c: Rgb, glyph: []const u8) void {
    var b: [64]u8 = undefined;
    const s = std.fmt.bufPrint(
        &b,
        "\x1b[{d};{d}H\x1b[38;2;{d};{d};{d}m{s}",
        .{ y + 1, x + 1, c.r, c.g, c.b, glyph },
    ) catch return;
    out(s);
}

pub fn run() void {
    const act = linux.Sigaction{
        .handler = .{ .handler = onSignal },
        .mask = std.mem.zeroes(linux.sigset_t),
        .flags = 0,
    };
    _ = linux.sigaction(linux.SIG.TERM, &act, null);
    _ = linux.sigaction(linux.SIG.INT, &act, null);

    var ws = termSize();
    var cols: i32 = ws.col;
    var rows: i32 = ws.row;

    var seed: u64 = undefined;
    _ = linux.getrandom(@ptrCast(&seed), @sizeOf(u64), 0);
    var prng = std.Random.DefaultPrng.init(seed);
    const rnd = prng.random();

    // enter alt screen, hide cursor, clear
    out("\x1b[?1049h\x1b[?25l\x1b[2J");
    defer out("\x1b[0m\x1b[?25h\x1b[?1049l");

    const max_pipes = 8;
    var pipes: [max_pipes]Pipe = undefined;
    var npipes = rnd.intRangeAtMost(usize, 2, max_pipes);
    for (pipes[0..npipes]) |*p| spawn(p, cols, rows, rnd);

    var cells: u64 = @intCast(cols * rows);
    var drawn: u64 = 0;

    while (running) {
        // Re-poll the terminal size every frame so we adapt to the fullscreen
        // transition that lands just *after* the daemon spawns us (and to any
        // later resize). Cheap ioctl; no SIGWINCH race.
        ws = termSize();
        if (ws.col != cols or ws.row != rows) {
            cols = ws.col;
            rows = ws.row;
            cells = @intCast(cols * rows);
            drawn = 0;
            out("\x1b[2J");
            npipes = rnd.intRangeAtMost(usize, 2, max_pipes);
            for (pipes[0..npipes]) |*p| spawn(p, cols, rows, rnd);
        }

        for (pipes[0..npipes]) |*p| {
            var new_dir = p.dir;
            // ~12% chance to turn 90° each step
            if (rnd.intRangeLessThan(u8, 0, 100) < 12) {
                new_dir = if (rnd.boolean()) (p.dir + 1) % 4 else (p.dir + 3) % 4;
            }
            draw(p.x, p.y, p.color, piece(p.dir, new_dir));
            p.dir = new_dir;

            switch (new_dir) {
                UP => p.y -= 1,
                DOWN => p.y += 1,
                LEFT => p.x -= 1,
                RIGHT => p.x += 1,
                else => {},
            }
            // wrap around edges
            if (p.x < 0) p.x = cols - 1;
            if (p.x >= cols) p.x = 0;
            if (p.y < 0) p.y = rows - 1;
            if (p.y >= rows) p.y = 0;
        }

        drawn += npipes;
        if (drawn > (cells * 9) / 10) { // screen mostly full -> reset, reshuffle
            drawn = 0;
            out("\x1b[2J");
            npipes = rnd.intRangeAtMost(usize, 2, max_pipes);
            for (pipes[0..npipes]) |*p| spawn(p, cols, rows, rnd);
        }

        sleepMs(45);
    }
}

fn spawn(p: *Pipe, cols: i32, rows: i32, rnd: std.Random) void {
    p.* = .{
        .x = rnd.intRangeLessThan(i32, 0, cols),
        .y = rnd.intRangeLessThan(i32, 0, rows),
        .dir = rnd.intRangeLessThan(u8, 0, 4),
        .color = palette[rnd.intRangeLessThan(usize, 0, palette.len)],
    };
}
